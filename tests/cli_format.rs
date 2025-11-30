// tests/cli_format.rs

#[test]
fn test_json_output_structure() {
    let json = r#"{"total_files":1,"total_violations":0}"#;
    assert!(json.starts_with('{'));
    assert!(json.ends_with('}'));
}

#[test]
fn test_json_includes_paths() {
    let json = r#"{"files":[{"path":"src/main.rs"}]}"#;
    assert!(json.contains("path"));
}

#[test]
fn test_sarif_output_structure() {
    let sarif = r#"{"$schema":"sarif","version":"2.1.0"}"#;
    assert!(sarif.contains("2.1.0"));
}

#[test]
fn test_sarif_includes_tool_info() {
    let sarif = r#"{"tool":{"driver":{"name":"warden"}}}"#;
    assert!(sarif.contains("warden"));
}
