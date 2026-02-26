// src/analysis/patterns/security_x02_test.rs

use super::*;
use tree_sitter::Parser;

fn parse_and_detect(code: &str) -> Vec<Violation> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_rust::language()).unwrap();
    let tree = parser.parse(code, None).unwrap();
    let mut violations = Vec::new();
    detect_x02_command(code, tree.root_node(), &mut violations);
    violations
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
fn is_shell_invocation_detects_shell_vars() {
    assert!(is_shell_invocation("sh", ""));
    assert!(is_shell_invocation("bash", ""));
    assert!(!is_shell_invocation("tailwind", ""));
    assert!(!is_shell_invocation("binary_path", ""));
}
