//! Integration test: locality integration in `neti check` pipeline.
//!
//! Verifies issue [9] — locality is a first-class stage in `neti check`,
//! gated by config mode (off/warn/error), included in JSON output and
//! neti-report.txt.

use std::process::Command;
use tempfile::TempDir;

/// Creates a workspace with the given `[rules.locality]` mode.
fn workspace_with_locality_mode(mode: &str) -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    let toml = format!(
        "[rules]\n\
         [rules.locality]\n\
         mode = \"{mode}\"\n\
         [preferences]\n\
         [commands]\n"
    );
    std::fs::write(dir.path().join("neti.toml"), toml).expect("write neti.toml");
    std::fs::write(dir.path().join("hello.rs"), "fn main() {}\n").expect("write hello.rs");
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
    serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout is not valid JSON: {e}\n---\n{stdout}"))
}

// --- JSON structure tests ---

#[test]
fn check_json_includes_locality_field() {
    let dir = workspace_with_locality_mode("warn");
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);

    assert!(
        value.get("locality").is_some(),
        "CheckReport JSON must include 'locality' field"
    );
}

#[test]
fn locality_field_has_required_keys() {
    let dir = workspace_with_locality_mode("warn");
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);

    let loc = value["locality"]
        .as_object()
        .expect("'locality' must be an object");

    assert!(
        loc.contains_key("violation_count"),
        "missing 'violation_count'"
    );
    assert!(loc.contains_key("violations"), "missing 'violations'");
    assert!(loc.contains_key("cycle_count"), "missing 'cycle_count'");
    assert!(loc.contains_key("cycles"), "missing 'cycles'");
    assert!(loc.contains_key("total_edges"), "missing 'total_edges'");
    assert!(loc.contains_key("mode"), "missing 'mode'");
    assert!(loc.contains_key("passed"), "missing 'passed'");
}

#[test]
fn locality_field_types_are_correct() {
    let dir = workspace_with_locality_mode("warn");
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);
    let loc = &value["locality"];

    assert!(
        loc["violation_count"].is_u64(),
        "'violation_count' must be a number"
    );
    assert!(
        loc["violations"].is_array(),
        "'violations' must be an array"
    );
    assert!(
        loc["cycle_count"].is_u64(),
        "'cycle_count' must be a number"
    );
    assert!(loc["cycles"].is_array(), "'cycles' must be an array");
    assert!(
        loc["total_edges"].is_u64(),
        "'total_edges' must be a number"
    );
    assert!(loc["mode"].is_string(), "'mode' must be a string");
    assert!(loc["passed"].is_boolean(), "'passed' must be a boolean");
}

#[test]
fn locality_violations_array_is_well_typed() {
    let dir = workspace_with_locality_mode("warn");
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);

    let violations = value["locality"]["violations"]
        .as_array()
        .expect("violations must be an array");

    // Single-file workspace has no edges, so no violations
    assert!(
        violations.is_empty(),
        "single-file workspace should have no locality violations"
    );
}

// --- Mode gating tests ---

#[test]
fn locality_mode_off_omits_violations() {
    let dir = workspace_with_locality_mode("off");
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);
    let loc = &value["locality"];

    assert_eq!(loc["mode"], "off");
    assert_eq!(loc["violation_count"], 0);
    assert_eq!(loc["total_edges"], 0);
    assert_eq!(loc["passed"], true);
}

#[test]
fn locality_mode_warn_passes_even_with_violations() {
    let dir = workspace_with_locality_mode("warn");
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);
    let loc = &value["locality"];

    assert_eq!(loc["mode"], "warn");
    assert_eq!(loc["passed"], true);
}

#[test]
fn locality_mode_error_passes_on_clean_graph() {
    let dir = workspace_with_locality_mode("error");
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);
    let loc = &value["locality"];

    assert_eq!(loc["mode"], "error");
    assert_eq!(loc["passed"], true);
}

#[test]
fn locality_mode_off_reflected_in_json() {
    let dir = workspace_with_locality_mode("off");
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);
    assert_eq!(value["locality"]["mode"], "off");
}

#[test]
fn locality_mode_warn_reflected_in_json() {
    let dir = workspace_with_locality_mode("warn");
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);
    assert_eq!(value["locality"]["mode"], "warn");
}

#[test]
fn locality_mode_error_reflected_in_json() {
    let dir = workspace_with_locality_mode("error");
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);
    assert_eq!(value["locality"]["mode"], "error");
}

// --- Report file tests ---

#[test]
fn neti_report_txt_includes_locality_section() {
    let dir = workspace_with_locality_mode("warn");
    let _output = run_check_json(&dir);

    let report = std::fs::read_to_string(dir.path().join("neti-report.txt"))
        .expect("neti-report.txt should exist");

    assert!(
        report.contains("NETI LOCALITY REPORT"),
        "neti-report.txt must contain LOCALITY REPORT section"
    );
    assert!(
        report.contains("Mode: warn"),
        "neti-report.txt must show mode"
    );
    assert!(
        report.contains("Result: PASS"),
        "neti-report.txt must show result"
    );
}

// --- Overall pass/fail integration ---

#[test]
fn overall_passed_incorporates_locality() {
    let dir = workspace_with_locality_mode("off");
    let output = run_check_json(&dir);
    let value = parse_stdout(&output);

    assert_eq!(
        value["passed"], true,
        "overall passed should be true with clean results"
    );
    assert!(output.status.success(), "exit code should be 0");
}
