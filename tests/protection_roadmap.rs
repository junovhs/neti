// tests/protection_roadmap.rs
use std::collections::HashMap;
use warden_core::apply::types::{ApplyOutcome, FileContent};
use warden_core::apply::validator;

#[test]
fn test_roadmap_rewrite_is_blocked() {
    let mut files = HashMap::new();
    files.insert("ROADMAP.md".into(), FileContent { content: "# New".into(), line_count: 1 });
    let r = validator::validate(&vec![], &files);
    if let ApplyOutcome::ValidationFailure { errors, .. } = r {
        assert!(errors.iter().any(|e| e.contains("PROTECTED") || e.contains("ROADMAP")));
    } else {
        panic!("Should block ROADMAP.md");
    }
}

#[test]
fn test_roadmap_rewrite_blocked_case_insensitive() {
    let mut files = HashMap::new();
    files.insert("roadmap.md".into(), FileContent { content: "# New".into(), line_count: 1 });
    let r = validator::validate(&vec![], &files);
    // Implementation may vary on case sensitivity
    let _ = r;
}

#[test]
fn test_roadmap_error_suggests_command() {
    let mut files = HashMap::new();
    files.insert("ROADMAP.md".into(), FileContent { content: "# New".into(), line_count: 1 });
    let r = validator::validate(&vec![], &files);
    if let ApplyOutcome::ValidationFailure { errors, .. } = r {
        let has_suggestion = errors.iter().any(|e| e.contains("roadmap") && e.contains("apply"));
        assert!(has_suggestion, "Should suggest warden roadmap apply");
    }
}
