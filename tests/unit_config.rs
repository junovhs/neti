// tests/unit_config.rs
use std::fs;
use warden_core::config::{Config, RuleConfig};

#[test]
fn test_load_toml() {
    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join("warden.toml"), "[rules]\nmax_file_tokens = 1500").unwrap();
    std::env::set_current_dir(d.path()).unwrap();
    let mut c = Config::new();
    c.load_local_config();
    assert_eq!(c.rules.max_file_tokens, 1500);
}

#[test]
fn test_defaults() {
    let r = RuleConfig::default();
    assert_eq!(r.max_file_tokens, 2000);
    assert_eq!(r.max_cyclomatic_complexity, 8);
    assert_eq!(r.max_nesting_depth, 3);
    assert_eq!(r.max_function_args, 5);
}

#[test]
fn test_command_single() {
    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join("warden.toml"), "[commands]\ncheck = \"cargo test\"").unwrap();
    std::env::set_current_dir(d.path()).unwrap();
    let mut c = Config::new();
    c.load_local_config();
    assert!(c.commands.contains_key("check"));
}

#[test]
fn test_command_list() {
    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join("warden.toml"), "[commands]\ncheck = [\"cargo clippy\", \"cargo test\"]").unwrap();
    std::env::set_current_dir(d.path()).unwrap();
    let mut c = Config::new();
    c.load_local_config();
    assert_eq!(c.commands.get("check").unwrap().len(), 2);
}

#[test]
fn test_wardenignore() {
    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join(".wardenignore"), "target\nnode_modules").unwrap();
    std::env::set_current_dir(d.path()).unwrap();
    let mut c = Config::new();
    c.load_local_config();
    assert!(!c.exclude_patterns.is_empty());
}

#[test]
fn test_ignore_tokens_on() {
    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join("warden.toml"), "[rules]\nignore_tokens_on = [\".md\"]").unwrap();
    std::env::set_current_dir(d.path()).unwrap();
    let mut c = Config::new();
    c.load_local_config();
    assert!(c.rules.ignore_tokens_on.iter().any(|s| s.contains("md")));
}

#[test]
fn test_ignore_naming_on() {
    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join("warden.toml"), "[rules]\nignore_naming_on = [\"tests\"]").unwrap();
    std::env::set_current_dir(d.path()).unwrap();
    let mut c = Config::new();
    c.load_local_config();
    assert!(c.rules.ignore_naming_on.contains(&"tests".to_string()));
}
