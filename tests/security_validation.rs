// tests/security_validation.rs
use std::collections::HashMap;
use warden_core::apply::types::{ApplyOutcome, FileContent};
use warden_core::apply::validator;

fn check(path: &str) -> bool {
    let mut files = HashMap::new();
    files.insert(path.into(), FileContent { content: "x".into(), line_count: 1 });
    matches!(validator::validate(&vec![], &files), ApplyOutcome::ValidationFailure { .. })
}

#[test]
fn test_traversal_blocked() { assert!(check("../etc/passwd")); }
#[test]
fn test_absolute_paths_blocked() { assert!(check("/etc/passwd")); }
#[test]
fn test_sensitive_paths_blocked() { assert!(check(".env")); }
#[test]
fn test_valid_paths_allowed() { assert!(!check("src/main.rs")); }
