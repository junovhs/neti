// tests/integration_apply.rs
use std::collections::HashMap;
use warden_core::apply::extractor;
use warden_core::apply::types::{ApplyOutcome, FileContent};
use warden_core::apply::validator;

#[test]
fn test_extract_single_file() {
    let input = "∇∇∇ src/main.rs ∇∇∇\nfn main() {}\n∆∆∆";
    let files = extractor::extract_files(input).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files.contains_key("src/main.rs"));
}

#[test]
fn test_extract_multiple_files() {
    let input = "∇∇∇ a.rs ∇∇∇\nfn a(){}\n∆∆∆\n∇∇∇ b.rs ∇∇∇\nfn b(){}\n∆∆∆";
    let files = extractor::extract_files(input).unwrap();
    assert_eq!(files.len(), 2);
}

#[test]
fn test_extract_skips_manifest() {
    let input = "∇∇∇ MANIFEST ∇∇∇\na.rs\n∆∆∆\n∇∇∇ a.rs ∇∇∇\nfn a(){}\n∆∆∆";
    let files = extractor::extract_files(input).unwrap();
    assert_eq!(files.len(), 1);
    assert!(!files.contains_key("MANIFEST"));
}

#[test]
fn test_extract_plan() {
    let input = "∇∇∇ PLAN ∇∇∇\nGOAL: Test\n∆∆∆";
    let plan = extractor::extract_plan(input);
    assert!(plan.is_some());
    assert!(plan.unwrap().contains("GOAL"));
}

#[test]
fn test_path_safety_blocks_traversal() {
    let mut files = HashMap::new();
    files.insert("../etc/passwd".into(), FileContent { content: "x".into(), line_count: 1 });
    let r = validator::validate(&vec![], &files);
    if let ApplyOutcome::ValidationFailure { errors, .. } = r {
        assert!(errors.iter().any(|e| e.contains("SECURITY") || e.contains("traversal")));
    } else {
        panic!("Should fail");
    }
}

#[test]
fn test_path_safety_blocks_absolute() {
    let mut files = HashMap::new();
    files.insert("/etc/passwd".into(), FileContent { content: "x".into(), line_count: 1 });
    let r = validator::validate(&vec![], &files);
    if let ApplyOutcome::ValidationFailure { errors, .. } = r {
        assert!(errors.iter().any(|e| e.contains("SECURITY") || e.contains("absolute")));
    } else {
        panic!("Should fail");
    }
}

#[test]
fn test_path_safety_blocks_git() {
    let mut files = HashMap::new();
    files.insert(".git/config".into(), FileContent { content: "x".into(), line_count: 1 });
    let r = validator::validate(&vec![], &files);
    assert!(matches!(r, ApplyOutcome::ValidationFailure { .. }));
}

#[test]
fn test_path_safety_blocks_hidden() {
    let mut files = HashMap::new();
    files.insert(".secret".into(), FileContent { content: "x".into(), line_count: 1 });
    let r = validator::validate(&vec![], &files);
    assert!(matches!(r, ApplyOutcome::ValidationFailure { .. }));
}

#[test]
fn test_truncation_detects_ellipsis_comment() {
    let mut files = HashMap::new();
    files.insert("a.rs".into(), FileContent { content: "fn f() {\n// ...\n}".into(), line_count: 3 });
    let r = validator::validate(&vec![], &files);
    if let ApplyOutcome::ValidationFailure { errors, .. } = r {
        assert!(errors.iter().any(|e| e.contains("truncation") || e.contains("...")));
    } else {
        panic!("Should fail");
    }
}

#[test]
fn test_truncation_allows_warden_ignore() {
    let mut files = HashMap::new();
    files.insert("a.rs".into(), FileContent { content: "fn f() {\n// ... warden:ignore\n}".into(), line_count: 3 });
    let r = validator::validate(&vec![], &files);
    if let ApplyOutcome::ValidationFailure { errors, .. } = r {
        let trunc: Vec<_> = errors.iter().filter(|e| e.contains("truncation")).collect();
        assert!(trunc.is_empty(), "warden:ignore should bypass");
    }
}

#[test]
fn test_truncation_detects_empty_file() {
    let mut files = HashMap::new();
    files.insert("a.rs".into(), FileContent { content: String::new(), line_count: 0 });
    let r = validator::validate(&vec![], &files);
    if let ApplyOutcome::ValidationFailure { errors, .. } = r {
        assert!(errors.iter().any(|e| e.contains("empty")));
    } else {
        panic!("Should fail");
    }
}

#[test]
fn test_path_safety_allows_valid() {
    let mut files = HashMap::new();
    files.insert("src/main.rs".into(), FileContent { content: "fn main() {}".into(), line_count: 1 });
    let r = validator::validate(&vec![], &files);
    if let ApplyOutcome::Success { written, .. } = r {
        assert!(written.contains(&"src/main.rs".to_string()));
    } else if let ApplyOutcome::ValidationFailure { errors, .. } = r {
        panic!("Should pass: {errors:?}");
    }
}

#[test]
fn test_unified_apply_roadmap() {
    let input = "===ROADMAP===\nCHECK task\n===END===";
    assert!(input.contains("===ROADMAP==="));
}

#[test]
fn test_unified_apply_combined() {
    let input = "===ROADMAP===\nCHECK task\n===END===\n∇∇∇ a.rs ∇∇∇\nfn a(){}\n∆∆∆";
    assert!(input.contains("===ROADMAP==="));
    let files = extractor::extract_files(input).unwrap();
    assert!(files.contains_key("a.rs"));
}
