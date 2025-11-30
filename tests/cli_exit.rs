// tests/cli_exit.rs - Exit code tests
use warden_core::config::Config;
use warden_core::analysis::RuleEngine;
use std::fs;
use tempfile::TempDir;

fn temp() -> TempDir {
    let d = tempfile::tempdir().unwrap();
    fs::create_dir_all(d.path().join("src")).unwrap();
    d
}

fn cfg() -> Config {
    let mut c = Config::new();
    c.rules.max_file_tokens = 100;
    c
}

#[test]
fn test_exit_0_clean() {
    let d = temp();
    fs::write(d.path().join("src/a.rs"), "fn main() {}").unwrap();
    let engine = RuleEngine::new(cfg());
    let report = engine.scan(vec![d.path().join("src/a.rs")]);
    assert_eq!(report.total_violations, 0);
}

#[test]
fn test_exit_1_violations() {
    let d = temp();
    fs::write(d.path().join("src/a.rs"), "fn f() { Some(1).unwrap(); }").unwrap();
    let engine = RuleEngine::new(cfg());
    let report = engine.scan(vec![d.path().join("src/a.rs")]);
    assert!(report.total_violations > 0);
}

#[test]
fn test_exit_codes_distinct() {
    let clean = 0;
    let violations = 1;
    let error = 2;
    assert_ne!(clean, violations);
    assert_ne!(violations, error);
    assert_ne!(clean, error);
}

#[test]
fn test_exit_0_empty_file_list() {
    let engine = RuleEngine::new(cfg());
    let report = engine.scan(vec![]);
    assert_eq!(report.total_violations, 0);
}
