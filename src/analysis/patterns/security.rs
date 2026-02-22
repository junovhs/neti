//! Security patterns: X01, X02, X03
//!
//! # X02 Design
//!
//! The original X02 rule flagged any `Command::new(variable)` as "command
//! injection." This is over-broad and generates false positives on idiomatic
//! `tokio::process::Command` usage in Dioxus/CLI tools.
//!
//! The real risk taxonomy for `Command::new`:
//!
//! **Shell injection (HIGH)** — when a shell interpreter is the executable
//! and user-controlled strings flow through as arguments.
//!
//! **Untrusted executable provenance (MEDIUM)** — when the executable name/path
//! comes from a variable whose origin we cannot verify. Direct `execve` with
//! `.arg(...)` is not injection, but PATH hijacking is still a concern.
//!
//! **Safe / no flag** — when the executable is a constant, comes from a
//! trusted config context, or is determined by an allowlist match.

use super::get_capture_node;
use crate::types::{Confidence, Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut out = Vec::new();
    detect_x01_sql(source, root, &mut out);
    detect_x02_command(source, root, &mut out);
    detect_x03_secrets(source, root, &mut out);
    out
}

// ── X01: SQL Injection ───────────────────────────────────────────────────────

fn detect_x01_sql(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r#"(macro_invocation macro: (identifier) @mac (token_tree) @args (#eq? @mac "format")) @fmt"#;
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else {
        return;
    };
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
                    },
                ));
            }
        }
    }
}

fn is_suspicious_sql(text: &str) -> bool {
    let upper = text.to_uppercase();
    let has_sql = upper.contains("SELECT ")
        || upper.contains("INSERT INTO ")
        || upper.contains("UPDATE ")
        || upper.contains("DELETE FROM ");
    let has_interp = text.contains("{}") || text.contains("{:");
    has_sql && has_interp
}

// ── X02: Command / Shell Injection ──────────────────────────────────────────

fn detect_x02_command(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r#"(call_expression
        function: (scoped_identifier path: (_) @path name: (identifier) @method)
        arguments: (arguments (identifier) @arg)
        (#eq? @method "new")) @call"#;

    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else {
        return;
    };
    let idx_call = query.capture_index_for_name("call");
    let idx_path = query.capture_index_for_name("path");
    let idx_arg = query.capture_index_for_name("arg");
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let call = get_capture_node(&m, idx_call);
        let path = get_capture_node(&m, idx_path);
        let arg = get_capture_node(&m, idx_arg);

        let (Some(call), Some(arg)) = (call, arg) else {
            continue;
        };

        let path_text = path
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .unwrap_or("");
        if !is_command_type(path_text) {
            continue;
        }

        let var_name = arg.utf8_text(source.as_bytes()).unwrap_or("");

        if is_safe_cmd_source(source, call, var_name) {
            continue;
        }

        let call_text = call.utf8_text(source.as_bytes()).unwrap_or("");
        if is_shell_invocation(var_name, call_text) {
            // HIGH: shell interpreter with dynamic args — provable injection vector
            out.push(x02_shell_injection(call.start_position().row + 1, var_name));
        } else {
            // MEDIUM: direct exec, no shell — risk is provenance, not injection
            out.push(x02_provenance_warn(call.start_position().row + 1, var_name));
        }
    }
}

/// Returns `true` if the scoped path indicates a `Command` type.
fn is_command_type(path: &str) -> bool {
    path.contains("Command") || path == "process" || path.ends_with("::process") || path.is_empty()
}

/// Returns `true` if this invocation directly runs a shell interpreter.
fn is_shell_invocation(var_name: &str, call_chain: &str) -> bool {
    let lower = var_name.to_lowercase();
    let shell_vars = [
        "sh",
        "bash",
        "dash",
        "zsh",
        "fish",
        "cmd",
        "powershell",
        "pwsh",
    ];
    if shell_vars.iter().any(|s| lower == *s) {
        return true;
    }
    if call_chain.contains(".arg(\"-c\")")
        || call_chain.contains(".arg(\"-c\" )")
        || call_chain.contains(".arg(\"/C\")")
        || call_chain.contains(".arg(\"--command\")")
    {
        return true;
    }
    false
}

fn x02_shell_injection(row: usize, var_name: &str) -> Violation {
    // HIGH confidence — shell interpreter with dynamic args is provably dangerous
    Violation::with_details(
        row,
        "Shell Injection risk: dynamic value used as shell executable".into(),
        "X02",
        ViolationDetails {
            function_name: None,
            analysis: vec![
                format!("Variable `{var_name}` becomes the shell binary."),
                "A shell as executable with dynamic args can run arbitrary code.".into(),
            ],
            suggestion: Some(
                "Use a const shell path, validate against an allowlist, or avoid shell -c.".into(),
            ),
        },
    )
}

fn x02_provenance_warn(row: usize, var_name: &str) -> Violation {
    // MEDIUM confidence — no shell, but provenance is unverifiable
    let mut v = Violation::with_details(
        row,
        format!("Untrusted executable provenance: `{var_name}` passed to Command::new"),
        "X02",
        ViolationDetails {
            function_name: None,
            analysis: vec![
                format!("The source of `{var_name}` is not verifiable at this site."),
                "Direct exec (no shell) is safe from injection, but PATH hijacking".into(),
                "or running an untrusted downloaded binary is a separate risk.".into(),
            ],
            suggestion: Some(
                "Prefer absolute paths, validate against an allowlist, or use a controlled install location.".into(),
            ),
        },
    );
    v.confidence = Confidence::Medium;
    v.confidence_reason =
        Some("direct exec without shell — risk is provenance, not injection".into());
    v
}

fn is_safe_cmd_source(source: &str, call: Node, var_name: &str) -> bool {
    if is_trusted_cmd_var(var_name) {
        return true;
    }
    if is_defined_const(source, var_name) {
        return true;
    }
    if is_config_context(source, call) {
        return true;
    }
    if var_name.contains('.') {
        let field = var_name.split('.').next_back().unwrap_or("");
        if is_trusted_cmd_var(field) {
            return true;
        }
    }
    is_in_match_allowlist(source, call)
}

fn is_trusted_cmd_var(name: &str) -> bool {
    const TRUSTED: &[&str] = &[
        "cmd",
        "command",
        "binary",
        "executable",
        "exe",
        "program",
        "prog",
        "shell",
        "interpreter",
        "compiler",
        "linker",
        "tool",
        "bin_path",
        "pbcopy",
        "pbpaste",
        "xclip",
        "xsel",
        "wl_copy",
        "wl_paste",
        "clip",
        "powershell",
        "osascript",
        "git",
        "cargo",
        "rustc",
        "tailwind",
        "npx",
        "node",
        "python",
        "python3",
    ];
    let lower = name.to_lowercase();
    TRUSTED
        .iter()
        .any(|&t| lower == t || lower.ends_with(&format!("_{t}")))
}

fn is_defined_const(source: &str, var_name: &str) -> bool {
    source.contains(&format!("const {var_name}")) || source.contains(&format!("static {var_name}"))
}

fn is_config_context(source: &str, node: Node) -> bool {
    let mut cur = node;
    for _ in 0..20 {
        let Some(p) = cur.parent() else { break };
        if p.kind() == "function_item" {
            if let Some(n) = p.child_by_field_name("name") {
                let fn_name = n.utf8_text(source.as_bytes()).unwrap_or("");
                let cfg = [
                    "parse",
                    "load",
                    "read",
                    "from_",
                    "deserialize",
                    "config",
                    "manifest",
                    "build",
                    "setup",
                ];
                if cfg.iter().any(|s| fn_name.contains(s)) {
                    return true;
                }
            }
        }
        cur = p;
    }
    false
}

fn is_in_match_allowlist(source: &str, node: Node) -> bool {
    let mut cur = node;
    for _ in 0..10 {
        let Some(p) = cur.parent() else { break };
        if p.kind() == "match_expression" {
            let text = p.utf8_text(source.as_bytes()).unwrap_or("");
            if text.contains('"') {
                return true;
            }
        }
        cur = p;
    }
    false
}

// ── X03: Hardcoded Secrets ───────────────────────────────────────────────────

fn detect_x03_secrets(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r#"
        (let_declaration pattern: (identifier) @name value: (string_literal) @value
            (#match? @name "(?i)(key|secret|token|password|auth)")) @decl
        (const_item name: (identifier) @name value: (string_literal) @value
            (#match? @name "(?i)(key|secret|token|password|auth)")) @const
    "#;

    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else {
        return;
    };
    let idx_value = query.capture_index_for_name("value");
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(val) = get_capture_node(&m, idx_value) {
            let text = val.utf8_text(source.as_bytes()).unwrap_or("");
            if text.contains("placeholder")
                || text.contains("example")
                || text.contains("test")
                || text.contains("dummy")
                || text.len() < 5
            {
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
                },
            ));
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_and_detect(code: &str) -> Vec<Violation> {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_rust::language()).unwrap();
        let tree = parser.parse(code, None).unwrap();
        detect(code, tree.root_node())
    }

    #[test]
    fn x02_direct_exec_with_args_is_provenance_not_injection() {
        let code = r#"
            async fn run_tailwind(binary_path: String) {
                tokio::process::Command::new(binary_path)
                    .arg("--input")
                    .arg("styles.css")
                    .spawn()
                    .unwrap();
            }
        "#;
        let vs = parse_and_detect(code);
        assert!(
            vs.iter().all(|v| !v.message.contains("Shell Injection")),
            "Direct exec with .arg() must not be classified as shell injection"
        );
    }

    #[test]
    fn x02_flags_shell_invocation() {
        let code = r#"
            fn run(cmd: String) {
                std::process::Command::new(sh)
                    .arg("-c")
                    .arg(&cmd)
                    .spawn().unwrap();
            }
        "#;
        let vs = parse_and_detect(code);
        assert!(
            vs.iter().any(|v| v.law == "X02"),
            "sh -c pattern should be flagged"
        );
    }

    #[test]
    fn x02_const_binary_is_safe() {
        let code = r#"
            const BINARY: &str = "/usr/bin/git";
            fn run() { std::process::Command::new(BINARY).spawn().unwrap(); }
        "#;
        let vs = parse_and_detect(code);
        assert!(
            vs.iter().all(|v| v.law != "X02"),
            "const binary should be safe"
        );
    }

    #[test]
    fn x01_flags_sql_format() {
        let code = r#"fn q(id: i32) { let _ = format!("SELECT * FROM users WHERE id = {}", id); }"#;
        assert!(parse_and_detect(code).iter().any(|v| v.law == "X01"));
    }

    #[test]
    fn is_shell_invocation_detects_shell_vars() {
        assert!(is_shell_invocation("sh", ""));
        assert!(is_shell_invocation("bash", ""));
        assert!(!is_shell_invocation("tailwind", ""));
        assert!(!is_shell_invocation("binary_path", ""));
    }
}
