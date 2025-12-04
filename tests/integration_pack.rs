use slopchop_core::config::Config;
use slopchop_core::pack::{self, PackOptions};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_nabla_delimiters_are_unique() {
    // Legacy name for roadmap compatibility. Tests SlopChop Protocol delimiters.
    let temp = tempdir().unwrap();
    let root = temp.path();
    let file_path = root.join("test.rs");
    fs::write(&file_path, "fn main() {}").unwrap();

    let config = Config::default();
    let opts = PackOptions {
        stdout: true,
        ..Default::default()
    };

    let content = pack::generate_content(&[file_path], &opts, &config).unwrap();

    assert!(content.contains("#__SLOPCHOP_FILE__#"));
    assert!(content.contains("#__SLOPCHOP_END__#"));
    // Verify old unicode symbols are gone
    assert!(!content.contains("∇∇∇"));
}

#[test]
fn test_nabla_format_structure() {
    // Legacy name for roadmap compatibility. Tests SlopChop Protocol format.
    let temp = tempdir().unwrap();
    let root = temp.path();
    let file_path = root.join("src/main.rs");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(&file_path, "code").unwrap();

    let config = Config::default();
    let opts = PackOptions::default();

    let content = pack::generate_content(std::slice::from_ref(&file_path), &opts, &config).unwrap();

    // Normalize path for test consistency
    let p_str = file_path.to_string_lossy().replace('\\', "/");
    let header = format!("#__SLOPCHOP_FILE__# {p_str}");

    assert!(content.contains(&header));
    assert!(content.contains("code"));
    assert!(content.contains("#__SLOPCHOP_END__#"));
}

#[test]
fn test_prompt_includes_nabla_instructions() {
    // Legacy name. Checks for SlopChop Protocol instructions.
    let config = Config::default();
    let generator = slopchop_core::prompt::PromptGenerator::new(config.rules);
    let prompt = generator.generate().unwrap();

    assert!(prompt.contains("#__SLOPCHOP_FILE__#"));
    assert!(prompt.contains("#__SLOPCHOP_MANIFEST__#"));
    assert!(prompt.contains("OUTPUT FORMAT (MANDATORY)"));
}

#[test]
fn test_prompt_includes_laws() {
    let config = Config::default();
    let generator = slopchop_core::prompt::PromptGenerator::new(config.rules);
    let prompt = generator.generate().unwrap();

    assert!(prompt.contains("THE 3 LAWS"));
    assert!(prompt.contains("LAW OF ATOMICITY"));
}

#[test]
fn test_prompt_includes_limits() {
    let config = Config::default();
    let generator = slopchop_core::prompt::PromptGenerator::new(config.rules);
    let prompt = generator.generate().unwrap();

    assert!(prompt.contains("Files: MUST be < 2000 tokens"));
    assert!(prompt.contains("Complexity: MUST be ≤ 8"));
}

#[test]
fn test_reminder_is_concise() {
    let config = Config::default();
    let generator = slopchop_core::prompt::PromptGenerator::new(config.rules);
    let reminder = generator.generate_reminder().unwrap();

    assert!(reminder.contains("SLOPCHOP CONSTRAINTS"));
    assert!(reminder.contains("#__SLOPCHOP_FILE__#"));
}

#[test]
fn test_pack_skeleton_integration() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    let file_path = root.join("test.rs");
    fs::write(&file_path, "fn main() { body }").unwrap();

    let config = Config::default();
    let opts = PackOptions {
        skeleton: true,
        ..Default::default()
    };

    let content = pack::generate_content(&[file_path], &opts, &config).unwrap();
    assert!(content.contains("fn main() { ... }"));
}

#[test]
fn test_smart_context_focus_mode() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    let target = root.join("target.rs");
    let other = root.join("other.rs");

    fs::write(&target, "fn target() { body }").unwrap();
    fs::write(&other, "fn other() { body }").unwrap();

    let config = Config::default();
    let opts = PackOptions {
        target: Some(target.clone()),
        ..Default::default()
    };

    let content = pack::generate_content(&[target, other], &opts, &config).unwrap();
    assert!(content.contains("fn target() { body }"));
    assert!(content.contains("fn other() { ... }"));
}
