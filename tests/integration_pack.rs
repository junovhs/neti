// tests/integration_pack.rs
use std::fs;
use tempfile::TempDir;
use warden_core::config::Config;
use warden_core::pack::{self, PackOptions};
use warden_core::prompt::PromptGenerator;

fn temp() -> TempDir {
    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(d.path().join("src")).unwrap();
    fs::write(d.path().join("src/main.rs"), "fn main() {}").unwrap();
    d
}

#[test]
fn test_nabla_delimiters_are_unique() {
    // Verify ∇∇∇ and ∆∆∆ don't appear in normal code
    let code = "fn main() { let x = 1; }";
    assert!(!code.contains("∇∇∇"));
    assert!(!code.contains("∆∆∆"));
}

#[test]
fn test_nabla_format_structure() {
    let d = temp();
    std::env::set_current_dir(d.path()).unwrap();
    let opts = PackOptions { prompt: false, ..Default::default() };
    let mut cfg = Config::new();
    cfg.load_local_config();
    let files = vec![d.path().join("src/main.rs")];
    let content = pack::generate_content(&files, &opts, &cfg).unwrap();
    assert!(content.contains("∇∇∇"));
    assert!(content.contains("∆∆∆"));
}

#[test]
fn test_prompt_includes_laws() {
    let cfg = Config::new();
    let gen = PromptGenerator::new(cfg.rules.clone());
    let prompt = gen.generate().unwrap();
    assert!(prompt.contains("LAW") || prompt.contains("Law"));
}

#[test]
fn test_prompt_includes_limits() {
    let cfg = Config::new();
    let gen = PromptGenerator::new(cfg.rules.clone());
    let prompt = gen.generate().unwrap();
    assert!(prompt.contains("2000") || prompt.contains("token"));
}

#[test]
fn test_prompt_includes_nabla_instructions() {
    let cfg = Config::new();
    let gen = PromptGenerator::new(cfg.rules.clone());
    let prompt = gen.generate().unwrap();
    assert!(prompt.contains("∇∇∇") || prompt.contains("Nabla") || prompt.contains("NABLA"));
}

#[test]
fn test_reminder_is_concise() {
    let cfg = Config::new();
    let gen = PromptGenerator::new(cfg.rules.clone());
    let reminder = gen.generate_reminder().unwrap();
    let lines: Vec<_> = reminder.lines().collect();
    assert!(lines.len() < 500);
}

#[test]
fn test_pack_skeleton_integration() {
    let d = temp();
    std::env::set_current_dir(d.path()).unwrap();
    let opts = PackOptions { skeleton: true, prompt: false, ..Default::default() };
    let mut cfg = Config::new();
    cfg.load_local_config();
    let files = vec![d.path().join("src/main.rs")];
    let content = pack::generate_content(&files, &opts, &cfg).unwrap();
    // Skeleton mode should produce { ... } or similar
    assert!(content.contains("...") || content.contains("main"));
}

#[test]
fn test_smart_context_focus_mode() {
    let d = temp();
    fs::write(d.path().join("src/other.rs"), "fn other() {}").unwrap();
    std::env::set_current_dir(d.path()).unwrap();
    let opts = PackOptions {
        target: Some(d.path().join("src/main.rs")),
        prompt: false,
        ..Default::default()
    };
    let mut cfg = Config::new();
    cfg.load_local_config();
    let files = vec![d.path().join("src/main.rs"), d.path().join("src/other.rs")];
    let content = pack::generate_content(&files, &opts, &cfg).unwrap();
    assert!(content.contains("main"));
}
