// tests/integration_apply.rs
// slopchop:ignore (test file - allowed .unwrap() for test clarity)
use slopchop_core::apply::types::{ManifestEntry, Operation};
use slopchop_core::apply::validator;
use std::collections::HashMap;

fn make_block(path: &str, content: &str) -> String {
    let header = format!("#__SLOPCHOP_FILE__# {path}");
    let footer = "#__SLOPCHOP_END__#";
    format!("{header}\n{content}\n{footer}\n")
}

fn make_manifest(entries: &[&str]) -> String {
    let header = "#__SLOPCHOP_MANIFEST__#";
    let footer = "#__SLOPCHOP_END__#";
    let body = entries.join("\n");
    format!("{header}\n{body}\n{footer}\n")
}

fn make_plan(goal: &str) -> String {
    let header = "#__SLOPCHOP_PLAN__#";
    let footer = "#__SLOPCHOP_END__#";
    format!("{header}\n{goal}\n{footer}\n")
}

#[test]
fn test_unified_apply_combined() {
    let manifest = make_manifest(&["src/main.rs", "src/lib.rs [NEW]"]);
    let block_main = make_block("src/main.rs", "fn main() {}");
    let block_lib = make_block("src/lib.rs", "pub fn lib() {}");
    let input = format!("{manifest}\n{block_main}\n{block_lib}");

    let manifest_parsed = slopchop_core::apply::manifest::parse_manifest(&input)
        .unwrap() // slopchop:ignore
        .unwrap(); // slopchop:ignore
    assert_eq!(manifest_parsed.len(), 2);
    assert_eq!(manifest_parsed[0].path, "src/main.rs");
    assert_eq!(manifest_parsed[1].path, "src/lib.rs");

    let files = slopchop_core::apply::extractor::extract_files(&input).unwrap(); // slopchop:ignore
    assert_eq!(files.len(), 2);
    assert!(files.contains_key("src/main.rs"));
    assert!(files.contains_key("src/lib.rs"));
}

#[test]
fn test_path_safety_blocks_traversal() {
    let manifest = vec![ManifestEntry {
        path: "../evil.rs".to_string(),
        operation: Operation::New,
    }];
    let extracted = HashMap::new();

    let outcome = validator::validate(&manifest, &extracted);
    if let slopchop_core::apply::types::ApplyOutcome::ValidationFailure { errors, .. } = outcome {
        assert!(errors.iter().any(|e| e.contains("Path traversal not allowed")));
    } else {
        panic!("Should have failed validation");
    }
}

#[test]
fn test_path_safety_blocks_absolute() {
    let manifest = vec![ManifestEntry {
        path: "/etc/passwd".to_string(),
        operation: Operation::New,
    }];
    let extracted = HashMap::new();

    let outcome = validator::validate(&manifest, &extracted);
    if let slopchop_core::apply::types::ApplyOutcome::ValidationFailure { errors, .. } = outcome {
        assert!(errors.iter().any(|e| e.contains("Absolute paths not allowed")));
    } else {
        panic!("Should have failed validation");
    }
}

#[test]
fn test_path_safety_blocks_hidden() {
    let manifest = vec![ManifestEntry {
        path: ".env".to_string(),
        operation: Operation::New,
    }];
    let extracted = HashMap::new();

    let outcome = validator::validate(&manifest, &extracted);
    if let slopchop_core::apply::types::ApplyOutcome::ValidationFailure { errors, .. } = outcome {
        assert!(errors.iter().any(|e| e.contains("sensitive") || e.contains("Hidden")));
    } else {
        panic!("Should have failed validation");
    }
}

#[test]
fn test_path_safety_blocks_git() {
    let manifest = vec![ManifestEntry {
        path: ".git/config".to_string(),
        operation: Operation::New,
    }];
    let extracted = HashMap::new();

    let outcome = validator::validate(&manifest, &extracted);
    if let slopchop_core::apply::types::ApplyOutcome::ValidationFailure { errors, .. } = outcome {
        assert!(errors.iter().any(|e| e.contains("sensitive") || e.contains(".git")));
    } else {
        panic!("Should have failed validation");
    }
}

#[test]
fn test_truncation_detects_ellipsis_comment() {
    use slopchop_core::apply::types::FileContent;
    let manifest = vec![ManifestEntry {
        path: "src/lib.rs".to_string(),
        operation: Operation::Update,
    }];
    let mut extracted = HashMap::new();
    let dots = "...";
    extracted.insert(
        "src/lib.rs".to_string(),
        FileContent {
            content: format!("fn main() {{\n    // {dots}\n}}"),
            line_count: 3,
        },
    );

    let outcome = validator::validate(&manifest, &extracted);
    if let slopchop_core::apply::types::ApplyOutcome::ValidationFailure { errors, .. } = outcome {
        assert!(errors.iter().any(|e| e.contains("Truncation")));
    } else {
        panic!("Should have detected truncation");
    }
}

#[test]
fn test_truncation_allows_slopchop_ignore() {
    use slopchop_core::apply::types::FileContent;
    let manifest = vec![ManifestEntry {
        path: "src/lib.rs".to_string(),
        operation: Operation::Update,
    }];
    let mut extracted = HashMap::new();
    let dots = "...";
    extracted.insert(
        "src/lib.rs".to_string(),
        FileContent {
            content: format!("fn main() {{\n    // {dots} slopchop:ignore\n}}"),
            line_count: 3,
        },
    );

    let outcome = validator::validate(&manifest, &extracted);
    match outcome {
        slopchop_core::apply::types::ApplyOutcome::Success { .. } => {}
        _ => panic!("Should have allowed slopchop:ignore"),
    }
}

#[test]
fn test_truncation_detects_empty_file() {
    use slopchop_core::apply::types::FileContent;
    let manifest = vec![ManifestEntry {
        path: "src/empty.rs".to_string(),
        operation: Operation::Update,
    }];
    let mut extracted = HashMap::new();
    extracted.insert(
        "src/empty.rs".to_string(),
        FileContent {
            content: "   \n\n  ".to_string(),
            line_count: 3,
        },
    );

    let outcome = validator::validate(&manifest, &extracted);
    if let slopchop_core::apply::types::ApplyOutcome::ValidationFailure { errors, .. } = outcome {
        assert!(errors.iter().any(|e| e.contains("empty")));
    } else {
        panic!("Should have detected empty file");
    }
}

#[test]
fn test_path_safety_allows_valid() {
    use slopchop_core::apply::types::FileContent;
    let manifest = vec![ManifestEntry {
        path: "src/nested/deep/file.rs".to_string(),
        operation: Operation::Update,
    }];
    let mut extracted = HashMap::new();
    extracted.insert(
        "src/nested/deep/file.rs".to_string(),
        FileContent {
            content: "fn valid() {}".to_string(),
            line_count: 1,
        },
    );

    let outcome = validator::validate(&manifest, &extracted);
    match outcome {
        slopchop_core::apply::types::ApplyOutcome::Success { .. } => {}
        slopchop_core::apply::types::ApplyOutcome::ValidationFailure { errors, .. } => {
            panic!("Should have passed: {errors:?}");
        }
        _ => panic!("Unexpected outcome"),
    }
}

#[test]
fn test_extract_plan() {
    let input = make_plan("GOAL: Fix bugs\nCHANGES:\n1. Fix thing");
    let plan = slopchop_core::apply::extractor::extract_plan(&input);
    assert!(plan.is_some());
    let p = plan.unwrap_or_default(); // slopchop:ignore
    assert!(p.contains("GOAL:"));
}

#[test]
fn test_extract_single_file() {
    let input = make_block("src/lib.rs", "fn hello() {}");
    let files = slopchop_core::apply::extractor::extract_files(&input).unwrap_or_default(); // slopchop:ignore
    assert_eq!(files.len(), 1);
    assert!(files.contains_key("src/lib.rs"));
}

#[test]
fn test_extract_multiple_files() {
    let b1 = make_block("src/a.rs", "fn a() {}");
    let b2 = make_block("src/b.rs", "fn b() {}");
    let input = format!("{b1}\n{b2}");
    let files = slopchop_core::apply::extractor::extract_files(&input).unwrap_or_default(); // slopchop:ignore
    assert_eq!(files.len(), 2);
}

#[test]
fn test_extract_skips_manifest() {
    let manifest = make_manifest(&["src/lib.rs"]);
    let block = make_block("src/lib.rs", "fn lib() {}");
    let input = format!("{manifest}\n{block}");
    let files = slopchop_core::apply::extractor::extract_files(&input).unwrap_or_default(); // slopchop:ignore
    assert_eq!(files.len(), 1);
    assert!(!files.contains_key("MANIFEST"));
}

#[test]
fn test_unified_apply_roadmap() {
    let input = "===ROADMAP===\nCHECK\nid = test-task\n===ROADMAP===";
    let cmds = slopchop_core::roadmap_v2::parser::parse_commands(input).unwrap_or_default(); // slopchop:ignore
    assert_eq!(cmds.len(), 1);
}

#[test]
fn test_delete_marker_detection() {
    let manifest = make_manifest(&["src/old.rs [DELETE]"]);
    let parsed = slopchop_core::apply::manifest::parse_manifest(&manifest)
        .ok()
        .flatten()
        .unwrap_or_default(); // slopchop:ignore
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].operation, Operation::Delete);
}

#[test]
fn test_delete_operation() {
    use slopchop_core::apply::writer::write_files;
    use slopchop_core::apply::types::FileContent;
    use tempfile::TempDir;
    use std::fs;

    let temp = TempDir::new().unwrap_or_else(|_| panic!("tempdir")); // slopchop:ignore
    let root = temp.path();
    fs::write(root.join("to_delete.rs"), "old").unwrap_or_default(); // slopchop:ignore

    let manifest = vec![ManifestEntry {
        path: "to_delete.rs".to_string(),
        operation: Operation::Delete,
    }];
    let files: HashMap<String, FileContent> = HashMap::new();

    let result = write_files(&manifest, &files, Some(root), 5).unwrap_or_else(|_| panic!("write")); // slopchop:ignore
    assert!(!root.join("to_delete.rs").exists());
    match result {
        slopchop_core::apply::types::ApplyOutcome::Success { deleted, .. } => {
            assert!(deleted.contains(&"to_delete.rs".to_string()));
        }
        _ => panic!("Expected success"),
    }
}

#[test]
fn test_block_windows_absolute() {
    let manifest = vec![ManifestEntry {
        path: "C:\\Windows\\System32\\evil.dll".to_string(),
        operation: Operation::New,
    }];
    let extracted = HashMap::new();

    let outcome = validator::validate(&manifest, &extracted);
    match outcome {
        slopchop_core::apply::types::ApplyOutcome::ValidationFailure { .. } => {}
        _ => panic!("Should block Windows absolute paths"),
    }
}

#[test]
fn test_block_backup_directory() {
    let manifest = vec![ManifestEntry {
        path: ".slopchop_apply_backup/secret.rs".to_string(),
        operation: Operation::New,
    }];
    let extracted = HashMap::new();

    let outcome = validator::validate(&manifest, &extracted);
    if let slopchop_core::apply::types::ApplyOutcome::ValidationFailure { errors, .. } = outcome {
        assert!(errors.iter().any(|e| e.contains("sensitive") || e.contains("backup")));
    } else {
        panic!("Should block backup directory access");
    }
}

#[test]
fn test_rejects_markdown_fences() {
    use slopchop_core::apply::types::FileContent;

    // Test backtick fences in non-markdown file
    let manifest = vec![ManifestEntry {
        path: "src/lib.rs".to_string(),
        operation: Operation::Update,
    }];
    let mut extracted = HashMap::new();

    // Use raw bytes to create the fence without triggering self-rejection
    let backtick_fence = String::from_utf8(vec![0x60, 0x60, 0x60]).unwrap_or_default(); // slopchop:ignore
    extracted.insert(
        "src/lib.rs".to_string(),
        FileContent {
            content: format!("{backtick_fence}rust\nfn main() {{}}\n{backtick_fence}"),
            line_count: 3,
        },
    );

    let outcome = validator::validate(&manifest, &extracted);
    if let slopchop_core::apply::types::ApplyOutcome::ValidationFailure { errors, .. } = outcome {
        assert!(
            errors.iter().any(|e| e.contains("Markdown fences")),
            "Should reject markdown fences in .rs file: {errors:?}"
        );
    } else {
        panic!("Should have rejected markdown fences in non-markdown file");
    }

    // Test that markdown files ARE allowed to have fences
    let manifest_md = vec![ManifestEntry {
        path: "README.md".to_string(),
        operation: Operation::Update,
    }];
    let mut extracted_md = HashMap::new();
    extracted_md.insert(
        "README.md".to_string(),
        FileContent {
            content: format!("# Title\n{backtick_fence}rust\nfn main() {{}}\n{backtick_fence}"),
            line_count: 4,
        },
    );

    let outcome_md = validator::validate(&manifest_md, &extracted_md);
    match outcome_md {
        slopchop_core::apply::types::ApplyOutcome::Success { .. } => {}
        slopchop_core::apply::types::ApplyOutcome::ValidationFailure { errors, .. } => {
            panic!("Markdown files should allow fences: {errors:?}");
        }
        _ => panic!("Unexpected outcome for markdown file"),
    }
}