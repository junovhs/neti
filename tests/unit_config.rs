// tests/unit_config.rs
use slopchop_core::config::Config;

#[test]
fn test_load_toml() {
    let toml = r#"
        [rules]
        max_file_tokens = 3000
        ignore_naming_on = ["foo"]

        [preferences]
        auto_copy = false
    "#;

    let mut config = Config::new();
    config.parse_toml(toml);

    assert_eq!(config.rules.max_file_tokens, 3000);
    assert!(config.rules.ignore_naming_on.contains(&"foo".to_string()));
    assert!(!config.preferences.auto_copy);
}

#[test]
fn test_defaults() {
    let config = Config::new();
    assert_eq!(config.rules.max_file_tokens, 2000);
    assert_eq!(config.rules.max_cyclomatic_complexity, 8);
    assert!(config.preferences.auto_copy);
}

#[test]
fn test_command_single() {
    let toml = r#"
        [commands]
        check = "cargo check"
    "#;
    let mut config = Config::new();
    config.parse_toml(toml);

    let cmds = config.commands.get("check").expect("check command missing");
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0], "cargo check");
}

#[test]
fn test_command_list() {
    let toml = r#"
        [commands]
        check = ["cargo fmt", "cargo test"]
    "#;
    let mut config = Config::new();
    config.parse_toml(toml);

    let cmds = config.commands.get("check").expect("check command missing");
    assert_eq!(cmds.len(), 2);
    assert_eq!(cmds[0], "cargo fmt");
    assert_eq!(cmds[1], "cargo test");
}

#[test]
fn test_slopchopignore() {
    let mut config = Config::new();

    // Should be ignored
    config.process_ignore_line("target");
    // SlopChop uses Regex for ignore patterns, not globs.
    // ".*\.log" matches any characters followed by .log
    config.process_ignore_line(r".*\.log");

    // Should be skipped
    config.process_ignore_line("# comment");
    config.process_ignore_line("");

    assert!(config.exclude_patterns.iter().any(|r| r.is_match("target")));
    assert!(config
        .exclude_patterns
        .iter()
        .any(|r| r.is_match("app.log")));
    assert!(!config.exclude_patterns.iter().any(|r| r.is_match("src")));
}

#[test]
fn test_ignore_tokens_on() {
    // Default includes "lock" and "README.md"
    let config = Config::new();
    let rules = config.rules;

    let is_ignored = |path: &str| rules.ignore_tokens_on.iter().any(|p| path.contains(p));

    assert!(is_ignored("Cargo.lock"));
    assert!(is_ignored("README.md"));
    assert!(!is_ignored("src/main.rs"));
}

#[test]
fn test_ignore_naming_on() {
    let toml = r#"
        [rules]
        ignore_naming_on = ["tests", "spec"]
    "#;
    let mut config = Config::new();
    config.parse_toml(toml);

    let is_ignored = |path: &str| {
        config
            .rules
            .ignore_naming_on
            .iter()
            .any(|p| path.contains(p))
    };

    assert!(is_ignored("tests/my_test.rs"));
    assert!(is_ignored("src/spec.rs"));
    assert!(!is_ignored("src/main.rs"));
}
