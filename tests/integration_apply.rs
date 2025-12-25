// tests/integration_apply.rs
// slopchop:ignore (test file - allowed .unwrap() for test clarity)
use slopchop_core::apply::types::{Block, FileContent, ManifestEntry, Operation};
use slopchop_core::apply::{manifest, parser, validator};
use std::collections::HashMap;

const SIGIL: &str = "XSC7XSC";

// --- Helpers ---

fn make_block(path: &str, content: &str) -> String {
    let header = format!("{SIGIL} FILE {SIGIL} {path}");
    let footer = format!("{SIGIL} END {SIGIL}");
    format!("{header}\n{content}\n{footer}\n")
}

fn make_manifest(entries: &[&str]) -> String {
    let header = format!("{SIGIL} MANIFEST {SIGIL}");
    let footer = format!("{SIGIL} END {SIGIL}");
    let body = entries.join("\n");
    format!("{header}\n{body}\n{footer}\n")
}

fn make_plan(goal: &str) -> String {
    let header = format!("{SIGIL} PLAN {SIGIL}");
    let footer = format!("{SIGIL} END {SIGIL}");
    format!("{header}\n{goal}\n{footer}\n")
}

fn parse_blocks_helper(input: &str) -> Vec<Block> {
    parser::parse(input).unwrap_or_default()
}

fn extract_files_helper(input: &str) -> HashMap<String, FileContent> {
    let mut files = HashMap::new();
    for block in parse_blocks_helper(input) {
        if let Block::File { path, content } = block {
            files.insert(
                path,
                FileContent {
                    line_count: content.lines().count(),
                    content,
                },
            );
        }
    }
    files
}

fn extract_plan_helper(input: &str) -> Option<String> {
    parse_blocks_helper(input).into_iter().find_map(|b| match b {
        Block::Plan(s) => Some(s),
        _ => None,
    })
}

fn extract_manifest_helper(input: &str) -> Option<Vec<ManifestEntry>> {
    let content = parse_blocks_helper(input).into_iter().find_map(|b| match b {
        Block::Manifest(s) => Some(s),
        _ => None,
    })?;
    manifest::parse_manifest_body(&content).ok()
}

// --- Tests ---

fn assert_security_rejection(path: &str, error_part: &str) {
    let manifest = vec![ManifestEntry {
        path: path.to_string(),
        operation: Operation::New,
    }];
    let extracted = HashMap::new();

    let outcome = validator::validate(&manifest, &extracted);
    if let slopchop_core::apply::types::ApplyOutcome::ValidationFailure { errors, .. } = outcome {
        assert!(
            errors.iter().any(|e| e.contains(error_part)),
            "Expected error containing '{error_part}' for path '{path}', got: {errors:?}"
        );
    } else {
        panic!("Should have failed validation for path: {path}");
    }
}

#[test]
fn test_security_boundaries() {
    let cases = vec![
        ("../evil.rs", "Path traversal"),
        ("/etc/passwd", "Absolute paths"),
        ("C:\\Windows\\System32\\evil.dll", "Absolute paths"),
        (".env", "sensitive"),
        (".git/config", "sensitive"),
        (".slopchop_apply_backup/secret.rs", "backup"),
    ];

    for (path, err) in cases {
        assert_security_rejection(path, err);
    }
}

#[test]
fn test_unified_apply_combined() {
    let manifest = make_manifest(&["src/main.rs", "src/lib.rs [NEW]"]);
    let block_main = make_block("src/main.rs", "fn main() {}");
    let block_lib = make_block("src/lib.rs", "pub fn lib() {}");
    let input = format!("{manifest}\n{block_main}\n{block_lib}");

    let manifest_parsed = extract_manifest_helper(&input).unwrap(); // slopchop:ignore
    assert_eq!(manifest_parsed.len(), 2);
    assert_eq!(manifest_parsed[0].path, "src/main.rs");
    assert_eq!(manifest_parsed[1].path, "src/lib.rs");

    let files = extract_files_helper(&input);
    assert_eq!(files.len(), 2);
    assert!(files.contains_key("src/main.rs"));
    assert!(files.contains_key("src/lib.rs"));
}

#[test]
fn test_truncation_detects_ellipsis_comment() {
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
    let plan = extract_plan_helper(&input);
    assert!(plan.is_some());
    let p = plan.unwrap_or_default(); // slopchop:ignore
    assert!(p.contains("GOAL:"));
}

#[test]
fn test_extract_single_file() {
    let input = make_block("src/lib.rs", "fn hello() {}");
    let files = extract_files_helper(&input);
    assert_eq!(files.len(), 1);
    assert!(files.contains_key("src/lib.rs"));
}

#[test]
fn test_extract_multiple_files() {
    let b1 = make_block("src/a.rs", "fn a() {}");
    let b2 = make_block("src/b.rs", "fn b() {}");
    let input = format!("{b1}\n{b2}");
    let files = extract_files_helper(&input);
    assert_eq!(files.len(), 2);
}

#[test]
fn test_extract_skips_manifest() {
    let manifest = make_manifest(&["src/lib.rs"]);
    let block = make_block("src/lib.rs", "fn lib() {}");
    let input = format!("{manifest}\n{block}");
    let files = extract_files_helper(&input);
    // The parser returns Blocks. Block::Manifest is distinct from Block::File.
    // extract_files_helper only collects Block::File.
    // So "MANIFEST" never appears as a file key unless it was enclosed in FILE block (which parser forbids).
    assert_eq!(files.len(), 1);
    assert!(!files.contains_key("MANIFEST"));
}

#[test]
fn test_delete_marker_detection() {
    let manifest = make_manifest(&["src/old.rs [DELETE]"]);
    let parsed = extract_manifest_helper(&manifest).unwrap_or_default(); // slopchop:ignore
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].operation, Operation::Delete);
}

#[test]
fn test_delete_operation() {
    use slopchop_core::apply::writer::write_files;
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
fn test_rejects_markdown_fences() {
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