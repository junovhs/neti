// tests/integration_audit.rs
use slopchop_core::roadmap::audit::{scan, AuditOptions, ViolationReason};
use slopchop_core::roadmap::types::Roadmap;
use std::fs;
use tempfile::tempdir;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn make_roadmap_with_task(task_text: &str) -> Roadmap {
    // Parse helper is cleaner than manual construction
    let content = format!("# Test\n## Section\n- [x] {task_text}");
    Roadmap::parse(&content)
}

#[test]
fn test_scans_completed_only() {
    let r = Roadmap::parse("# T\n\n## v0.1.0\n\n- [x] Done\n- [ ] Todo\n");
    let report = scan(
        &r,
        std::path::Path::new("."),
        &AuditOptions { strict: false },
    );

    // Only "Done" should be checked. "Todo" ignored.
    // "Done" is not exempt, so it's checked.
    assert_eq!(report.total_checked, 1);
}

#[test]
fn test_no_test_skipped() {
    let r = make_roadmap_with_task("Manual check [no-test]");
    let report = scan(
        &r,
        std::path::Path::new("."),
        &AuditOptions { strict: false },
    );
    // [no-test] means it is skipped from checking.
    assert_eq!(report.total_checked, 0);
}

#[test]
fn test_explicit_anchor_verified() -> Result<()> {
    let d = tempdir()?;
    let root = d.path();

    // Create the test file
    let test_file = root.join("tests/feature.rs");
    fs::create_dir_all(test_file.parent().unwrap())?;

    // Use a function name that matches the feature slug "feature" -> "test_feature"
    // to satisfy strict naming conventions
    fs::write(&test_file, "fn test_feature() {}")?;

    let task_text = "Feature <!-- test: tests/feature.rs::test_feature -->";
    let r = make_roadmap_with_task(task_text);

    let report = scan(&r, root, &AuditOptions { strict: true });

    if !report.violations.is_empty() {
        println!("Violations found: {:?}", report.violations);
    }

    assert!(report.violations.is_empty(), "Should pass verification");
    assert_eq!(report.total_checked, 1);
    Ok(())
}

#[test]
fn test_missing_file_detected() {
    let d = tempdir().unwrap();
    let root = d.path();

    let task_text = "Ghost Feature <!-- test: tests/ghost.rs::boo -->";
    let r = make_roadmap_with_task(task_text);

    let report = scan(&r, root, &AuditOptions { strict: true });

    assert_eq!(report.violations.len(), 1);
    match &report.violations[0].reason {
        ViolationReason::MissingTestFile(f) => assert_eq!(f, "tests/ghost.rs"),
        _ => panic!("Expected MissingTestFile violation"),
    }
}
