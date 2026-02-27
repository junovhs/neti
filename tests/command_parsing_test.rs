//! Integration test: command parsing with shell-words.
//!
//! Verifies issue [28] — commands with quoted arguments, spaces,
//! and special characters are parsed correctly by the verification
//! runner. Tests invoke the compiled `neti` binary with `neti.toml`
//! configs containing quoted commands.

use std::process::Command;
use tempfile::TempDir;

/// Creates a temp workspace with a neti.toml containing the given check commands.
fn workspace_with_commands(commands: &[&str]) -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");

    let mut toml = String::from("[rules]\n[preferences]\n[commands]\ncheck = [\n");
    for cmd in commands {
        toml.push_str(&format!(
            "  \"{}\",\n",
            cmd.replace('\\', "\\\\").replace('"', "\\\"")
        ));
    }
    toml.push_str("]\n");

    std::fs::write(dir.path().join("neti.toml"), toml).expect("failed to write neti.toml");
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
fn simple_command_passes_in_pipeline() {
    let dir = workspace_with_commands(&["echo hello"]);
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);

    let commands = value["commands"].as_array().expect("commands array");
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0]["passed"], true);
}

#[test]
fn quoted_argument_preserved_in_pipeline() {
    // The command `echo "hello world"` should produce output containing "hello world"
    // If split_whitespace were used, echo would get "\"hello" and "world\"" separately
    let dir = workspace_with_commands(&["echo 'hello world'"]);
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);

    let commands = value["commands"].as_array().expect("commands array");
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0]["passed"], true);
    let stdout = commands[0]["stdout"].as_str().unwrap_or("");
    assert!(
        stdout.contains("hello world"),
        "quoted arg should be preserved, got: {stdout}"
    );
}

#[test]
fn failing_command_reported_in_json() {
    let dir = workspace_with_commands(&["false"]);
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);

    let commands = value["commands"].as_array().expect("commands array");
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0]["passed"], false);
    assert_eq!(value["passed"], false);
}

#[test]
fn multiple_commands_all_reported() {
    let dir = workspace_with_commands(&["echo first", "echo second"]);
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);

    let commands = value["commands"].as_array().expect("commands array");
    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0]["passed"], true);
    assert_eq!(commands[1]["passed"], true);
}

#[test]
fn command_result_has_required_fields() {
    let dir = workspace_with_commands(&["echo test"]);
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);

    let cmd = &value["commands"][0];
    let obj = cmd.as_object().expect("command result must be object");
    assert!(obj.contains_key("command"), "missing 'command'");
    assert!(obj.contains_key("passed"), "missing 'passed'");
    assert!(obj.contains_key("exit_code"), "missing 'exit_code'");
    assert!(obj.contains_key("stdout"), "missing 'stdout'");
    assert!(obj.contains_key("stderr"), "missing 'stderr'");
    assert!(obj.contains_key("duration_ms"), "missing 'duration_ms'");
}

#[test]
fn nonexistent_command_fails_gracefully() {
    let dir = workspace_with_commands(&["nonexistent_binary_xyz_123"]);
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);

    let commands = value["commands"].as_array().expect("commands array");
    assert_eq!(commands[0]["passed"], false);
    assert_eq!(commands[0]["exit_code"], -1);
}
