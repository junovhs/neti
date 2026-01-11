// src/analysis/v2/patterns/security.rs
//! Security patterns: X01, X02, X03

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut violations = Vec::new();
    detect_x01_sql(source, root, &mut violations);
    detect_x02_command(source, root, &mut violations);
    detect_x03_secrets(source, root, &mut violations);
    violations
}

fn detect_x01_sql(source: &str, root: Node, out: &mut Vec<Violation>) {
    let query_str = r#"
        (macro_invocation
            macro: (identifier) @mac
            (token_tree) @args
            (#eq? @mac "format")) @fmt
    "#;

    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else { return; };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(arg_node) = m.captures.iter().find(|c| c.index == 1).map(|c| c.node) {
            let args = arg_node.utf8_text(source.as_bytes()).unwrap_or("");
            if is_suspicious_sql_format(args) {
                let row = arg_node.start_position().row + 1;
                out.push(Violation::with_details(
                    row,
                    "Potential SQL Injection detected".to_string(),
                    "X01",
                    ViolationDetails {
                        function_name: None,
                        analysis: vec!["Formatting into SQL bypasses parameterization.".into()],
                        suggestion: Some("Use parameterized queries instead.".into()),
                    }
                ));
            }
        }
    }
}

fn is_suspicious_sql_format(text: &str) -> bool {
    let upper = text.to_uppercase();
    let has_sql = upper.contains("SELECT ")
        || upper.contains("INSERT INTO ")
        || upper.contains("UPDATE ")
        || upper.contains("DELETE FROM ");
    let has_interpolation = text.contains("{}") || text.contains("{:");
    has_sql && has_interpolation
}

fn detect_x02_command(source: &str, root: Node, out: &mut Vec<Violation>) {
    let query_str = r#"
        (call_expression
            function: (scoped_identifier
                path: (identifier) @struct
                name: (identifier) @method)
            arguments: (arguments
                (identifier) @arg)
            (#eq? @struct "Command")
            (#eq? @method "new")) @call
    "#;

    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else { return; };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let call_node = m.captures.iter()
            .find(|c| query.capture_names()[c.index as usize] == "call")
            .map(|c| c.node);
        let arg_node = m.captures.iter()
            .find(|c| query.capture_names()[c.index as usize] == "arg")
            .map(|c| c.node);

        let Some(call) = call_node else { continue };
        let Some(arg) = arg_node else { continue };

        let var_name = arg.utf8_text(source.as_bytes()).unwrap_or("");

        if is_safe_cmd_source(source, call, var_name) { continue; }

        let row = call.start_position().row + 1;
        out.push(Violation::with_details(
            row,
            "Potential Command Injection".to_string(),
            "X02",
            ViolationDetails {
                function_name: None,
                analysis: vec![
                    "Variable passed to `Command::new` without clear provenance.".into(),
                    format!("Variable `{var_name}` source is not statically verifiable."),
                ],
                suggestion: Some("Validate against allowlist or use a const.".into()),
            }
        ));
    }
}

fn is_safe_cmd_source(source: &str, call: Node, var_name: &str) -> bool {
    if is_trusted_cmd_var(var_name) { return true; }
    if is_defined_const(source, var_name) { return true; }
    if is_config_context(source, call) { return true; }

    if var_name.contains('.') {
        let field = var_name.split('.').next_back().unwrap_or("");
        if is_trusted_cmd_var(field) { return true; }
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
    source.contains(&format!("const {var_name}"))
        || source.contains(&format!("static {var_name}"))
}

fn is_config_context(source: &str, node: Node) -> bool {
    let mut current = node;
    for _ in 0..20 {
        if let Some(parent) = current.parent() {
            if parent.kind() == "function_item" {
                if let Some(name_node) = parent.child_by_field_name("name") {
                    let fn_name = name_node.utf8_text(source.as_bytes()).unwrap_or("");
                    let config_fns = ["parse", "load", "read", "from_", "deserialize",
                                      "config", "manifest", "settings"];
                    if config_fns.iter().any(|p| fn_name.contains(p)) { return true; }
                }
            }
            current = parent;
        } else { break; }
    }
    false
}

fn is_in_match_allowlist(source: &str, node: Node) -> bool {
    let mut current = node;
    for _ in 0..10 {
        if let Some(parent) = current.parent() {
            if parent.kind() == "match_expression" {
                let match_text = parent.utf8_text(source.as_bytes()).unwrap_or("");
                if match_text.contains('"') { return true; }
            }
            current = parent;
        } else { break; }
    }
    false
}

fn detect_x03_secrets(source: &str, root: Node, out: &mut Vec<Violation>) {
    let query_str = r#"
        (let_declaration
            pattern: (identifier) @name
            value: (string_literal) @value
            (#match? @name "(?i)(key|secret|token|password|auth)")) @decl
        (const_item
            name: (identifier) @name
            value: (string_literal) @value
            (#match? @name "(?i)(key|secret|token|password|auth)")) @const
    "#;

    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else { return; };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(val_node) = m.captures.iter().find(|c| c.index == 1).map(|c| c.node) {
            let val = val_node.utf8_text(source.as_bytes()).unwrap_or("");
            if val.contains("placeholder") || val.contains("example")
               || val.contains("test") || val.contains("dummy") || val.len() < 5 {
                continue;
            }

            let row = val_node.start_position().row + 1;
            out.push(Violation::with_details(
                row,
                "Potential hardcoded secret detected".to_string(),
                "X03",
                ViolationDetails {
                    function_name: None,
                    analysis: vec!["Hardcoded secrets should be loaded from environment.".into()],
                    suggestion: Some("Use `std::env::var()` or a secrets manager.".into()),
                }
            ));
        }
    }
}
