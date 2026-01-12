// src/analysis/v2/patterns/security.rs
//! Security patterns: X01, X02, X03

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};
use super::get_capture_node;

#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut out = Vec::new();
    detect_x01_sql(source, root, &mut out);
    detect_x02_command(source, root, &mut out);
    detect_x03_secrets(source, root, &mut out);
    out
}

fn detect_x01_sql(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r#"(macro_invocation macro: (identifier) @mac (token_tree) @args (#eq? @mac "format")) @fmt"#;
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else { return };
    let idx_args = query.capture_index_for_name("args");
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(arg_node) = get_capture_node(&m, idx_args) {
            let args = arg_node.utf8_text(source.as_bytes()).unwrap_or("");
            if is_suspicious_sql(args) {
                out.push(Violation::with_details(
                    arg_node.start_position().row + 1,
                    "Potential SQL Injection".into(),
                    "X01",
                    ViolationDetails {
                        function_name: None,
                        analysis: vec!["Formatting into SQL bypasses parameterization.".into()],
                        suggestion: Some("Use parameterized queries.".into()),
                    }
                ));
            }
        }
    }
}

fn is_suspicious_sql(text: &str) -> bool {
    let upper = text.to_uppercase();
    let has_sql = upper.contains("SELECT ") || upper.contains("INSERT INTO ")
        || upper.contains("UPDATE ") || upper.contains("DELETE FROM ");
    let has_interp = text.contains("{}") || text.contains("{:");
    has_sql && has_interp
}

fn detect_x02_command(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r#"(call_expression
        function: (scoped_identifier path: (identifier) @struct name: (identifier) @method)
        arguments: (arguments (identifier) @arg)
        (#eq? @struct "Command") (#eq? @method "new")) @call"#;

    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else { return };
    let idx_call = query.capture_index_for_name("call");
    let idx_arg = query.capture_index_for_name("arg");
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let call = get_capture_node(&m, idx_call);
        let arg = get_capture_node(&m, idx_arg);

        let (Some(call), Some(arg)) = (call, arg) else { continue };
        let var_name = arg.utf8_text(source.as_bytes()).unwrap_or("");

        if is_safe_cmd_source(source, call, var_name) { continue }

        out.push(Violation::with_details(
            call.start_position().row + 1,
            "Potential Command Injection".into(),
            "X02",
            ViolationDetails {
                function_name: None,
                analysis: vec![
                    "Variable passed to `Command::new` without clear provenance.".into(),
                    format!("Variable `{var_name}` source not verifiable."),
                ],
                suggestion: Some("Validate against allowlist or use const.".into()),
            }
        ));
    }
}

fn is_safe_cmd_source(source: &str, call: Node, var_name: &str) -> bool {
    if is_trusted_cmd_var(var_name) { return true }
    if is_defined_const(source, var_name) { return true }
    if is_config_context(source, call) { return true }
    if var_name.contains('.') {
        let field = var_name.split('.').next_back().unwrap_or("");
        if is_trusted_cmd_var(field) { return true }
    }
    is_in_match_allowlist(source, call)
}

fn is_trusted_cmd_var(name: &str) -> bool {
    let trusted = [
        "cmd", "command", "binary", "executable", "exe", "program", "prog",
        "shell", "interpreter", "compiler", "linker", "tool", "bin_path",
        "pbcopy", "pbpaste", "xclip", "xsel", "wl_copy", "wl_paste",
        "clip", "powershell", "osascript", "git", "cargo", "rustc",
    ];
    let lower = name.to_lowercase();
    trusted.iter().any(|&t| lower == t || lower.ends_with(&format!("_{t}")))
}

fn is_defined_const(source: &str, var_name: &str) -> bool {
    source.contains(&format!("const {var_name}")) || source.contains(&format!("static {var_name}"))
}

fn is_config_context(source: &str, node: Node) -> bool {
    let mut cur = node;
    for _ in 0..20 {
        if let Some(p) = cur.parent() {
            if p.kind() == "function_item" {
                if let Some(n) = p.child_by_field_name("name") {
                    let fn_name = n.utf8_text(source.as_bytes()).unwrap_or("");
                    let cfg = ["parse", "load", "read", "from_", "deserialize", "config", "manifest"];
                    if cfg.iter().any(|s| fn_name.contains(s)) { return true }
                }
            }
            cur = p;
        } else { break }
    }
    false
}

fn is_in_match_allowlist(source: &str, node: Node) -> bool {
    let mut cur = node;
    for _ in 0..10 {
        if let Some(p) = cur.parent() {
            if p.kind() == "match_expression" {
                let text = p.utf8_text(source.as_bytes()).unwrap_or("");
                if text.contains('"') { return true }
            }
            cur = p;
        } else { break }
    }
    false
}

fn detect_x03_secrets(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r#"
        (let_declaration pattern: (identifier) @name value: (string_literal) @value
            (#match? @name "(?i)(key|secret|token|password|auth)")) @decl
        (const_item name: (identifier) @name value: (string_literal) @value
            (#match? @name "(?i)(key|secret|token|password|auth)")) @const
    "#;

    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else { return };
    let idx_value = query.capture_index_for_name("value");
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(val) = get_capture_node(&m, idx_value) {
            let text = val.utf8_text(source.as_bytes()).unwrap_or("");
            if text.contains("placeholder") || text.contains("example")
               || text.contains("test") || text.contains("dummy") || text.len() < 5 {
                continue;
            }
            out.push(Violation::with_details(
                val.start_position().row + 1,
                "Potential hardcoded secret".into(),
                "X03",
                ViolationDetails {
                    function_name: None,
                    analysis: vec!["Secrets should come from environment.".into()],
                    suggestion: Some("Use `std::env::var()`.".into()),
                }
            ));
        }
    }
}