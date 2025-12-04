// tests/unit_validator.rs
use slopchop_core::apply::types::{ApplyOutcome, FileContent};
use slopchop_core::apply::validator;
use std::collections::HashMap;

fn check_content(content: &str) -> bool {
    let mut files = HashMap::new();
    files.insert(
        "test.rs".into(),
        FileContent {
            content: content.into(),
            line_count: 1,
        },
    );
    matches!(
        validator::validate(&vec![], &files),
        ApplyOutcome::ValidationFailure { .. }
    )
}

fn check_path(path: &str) -> bool {
    let mut files = HashMap::new();
    files.insert(
        path.into(),
        FileContent {
            content: "data".into(),
            line_count: 1,
        },
    );
    matches!(
        validator::validate(&vec![], &files),
        ApplyOutcome::ValidationFailure { .. }
    )
}

#[test]
fn test_block_comment_ellipsis() {
    assert!(check_content("/* ... */"));
}

#[test]
fn test_hash_ellipsis() {
    assert!(check_content("# ..."));
}

#[test]
fn test_lazy_phrase_rest_of() {
    assert!(check_content("// rest of implementation"));
}

#[test]
fn test_lazy_phrase_remaining() {
    assert!(check_content("// remaining code"));
}

#[test]
fn test_valid_code_passes() {
    assert!(!check_content("fn main() {}"));
}

#[test]
fn test_ellipsis_in_string_allowed() {
    assert!(!check_content("let s = \"Loading...\";"));
}

#[test]
fn test_slopchop_ignore_inline() {
    assert!(!check_content("// ... slopchop:ignore"));
}

#[test]
fn test_line_number_reported() {
    let content = "fn ok() {}\n// ...";
    let mut files = HashMap::new();
    files.insert(
        "test.rs".into(),
        FileContent {
            content: content.into(),
            line_count: 2,
        },
    );

    if let ApplyOutcome::ValidationFailure { errors, .. } = validator::validate(&vec![], &files) {
        // Line 2 should be cited (1-based index)
        assert!(errors.iter().any(|e| e.contains(":2:")));
    } else {
        panic!("Should have failed validation");
    }
}

#[test]
fn test_gnupg_blocked() {
    assert!(check_path(".gnupg/secring.gpg"));
}

#[test]
fn test_id_rsa_blocked() {
    assert!(check_path("ssh/id_rsa"));
}

#[test]
fn test_credentials_blocked() {
    assert!(check_path(".aws/credentials"));
}

#[test]
fn test_backup_dir_blocked() {
    assert!(check_path(".slopchop_apply_backup/foo"));
}
