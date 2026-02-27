//! Integration test: `neti check --json` must emit valid JSON to stdout.
//!
//! Verifies issue [26] — the `--json` flag on `neti check` emits a
//! `CheckReport` JSON payload to stdout containing scan results,
//! command results, and overall pass/fail status.
//!
//! Tests run in a temp directory with a minimal `neti.toml` to avoid
//! recursive invocation (the real config runs `cargo test` as a check
//! command, which would re-enter these tests).

use std::process::Command;
use tempfile::TempDir;

/// Creates a temp directory with a minimal Rust file and `neti.toml`
/// with no commands, so `neti check` completes quickly without recursion.
fn test_workspace() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::write(
        dir.path().join("neti.toml"),
        "[rules]\n[preferences]\n[commands]\n",
    )
    .expect("failed to write neti.toml");
    std::fs::write(dir.path().join("hello.rs"), "fn main() {}\n")
        .expect("failed to write hello.rs");
    dir
}

fn run_check_json(dir: &TempDir) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_neti"))
        .args(["check", "--json"])
        .current_dir(dir.path())
        .output()
        .expect("failed to execute neti")
}

fn parse_stdout(output: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("stdout is not valid JSON")
}

#[test]
fn check_json_emits_valid_json_to_stdout() {
    let dir = test_workspace();
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);

    let obj = value.as_object().expect("JSON root must be an object");
    assert!(obj.contains_key("scan"), "missing 'scan' field");
    assert!(obj.contains_key("commands"), "missing 'commands' field");
    assert!(obj.contains_key("passed"), "missing 'passed' field");
}

#[test]
fn check_json_scan_has_required_fields() {
    let dir = test_workspace();
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);

    let scan = value["scan"].as_object().expect("'scan' must be an object");
    assert!(scan.contains_key("files"), "scan missing 'files'");
    assert!(
        scan.contains_key("total_tokens"),
        "scan missing 'total_tokens'"
    );
    assert!(
        scan.contains_key("total_violations"),
        "scan missing 'total_violations'"
    );
    assert!(
        scan.contains_key("duration_ms"),
        "scan missing 'duration_ms'"
    );
}

#[test]
fn check_json_commands_is_empty_array_with_no_config() {
    let dir = test_workspace();
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);

    let commands = value["commands"]
        .as_array()
        .expect("'commands' must be an array");
    assert!(
        commands.is_empty(),
        "commands should be empty when neti.toml has no check commands"
    );
}

#[test]
fn check_json_passed_is_boolean() {
    let dir = test_workspace();
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);

    assert!(value["passed"].is_boolean(), "'passed' must be a boolean");
}

#[test]
fn check_json_exit_code_matches_passed() {
    let dir = test_workspace();
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);

    if value["passed"].as_bool() == Some(true) {
        assert!(
            output.status.success(),
            "exit code should be 0 when passed=true"
        );
    } else {
        assert!(
            !output.status.success(),
            "exit code should be non-zero when passed=false"
        );
    }
}
