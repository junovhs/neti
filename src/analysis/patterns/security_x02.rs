// src/analysis/patterns/security_x02.rs
//! X02: Command / Shell Injection.
//!
//! **Shell injection (HIGH)** — shell interpreter as executable with dynamic args.
//! **Untrusted executable provenance (MEDIUM)** — executable from unverified variable.
//! **Safe / no flag** — const, config context, or allowlist match.

use super::super::get_capture_node;
use crate::types::{Confidence, Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

#[cfg(test)]
#[path = "security_x02_test.rs"]
mod tests;

pub(super) fn detect_x02_command(source: &str, root: Node, out: &mut Vec<Violation>) {
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
            out.push(x02_shell_injection(call.start_position().row + 1, var_name));
        } else {
            out.push(x02_provenance_warn(call.start_position().row + 1, var_name));
        }
    }
}

/// Returns `true` if the scoped path indicates a `Command` type.
fn is_command_type(path: &str) -> bool {
    path.contains("Command") || path == "process" || path.ends_with("::process") || path.is_empty()
}

/// Returns `true` if this invocation directly runs a shell interpreter.
pub(super) fn is_shell_invocation(var_name: &str, call_chain: &str) -> bool {
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
