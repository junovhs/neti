// tests/unit_audit.rs
use slopchop_core::roadmap::audit::{scan, AuditOptions, ViolationReason};
use slopchop_core::roadmap::types::{Roadmap, Task, TaskStatus};
use std::fs;
use tempfile::tempdir;

fn make_task(id: &str, tests: Vec<String>) -> Task {
    Task {
        id: id.into(),
        path: format!("section/{id}"),
        text: format!("Task {id}"),
        status: TaskStatus::Complete,
        indent: 0,
        line: 0,
        children: vec![],
        tests,
    }
}

fn make_roadmap(tasks: Vec<Task>) -> Roadmap {
    use slopchop_core::roadmap::types::Section;

    let section = Section {
        id: "main".into(),
        heading: "Main".into(),
        level: 2,
        theme: None,
        tasks,
        subsections: vec![],
        raw_content: String::new(),
        line_start: 0,
        line_end: 0,
    };

    Roadmap {
        path: None,
        title: "Test Roadmap".into(),
        sections: vec![section],
        raw: String::new(),
    }
}

#[test]
fn test_missing_file_detection() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let task = make_task("t1", vec!["missing_file.rs".into()]);
    let roadmap = make_roadmap(vec![task]);
    let opts = AuditOptions { strict: true };

    let report = scan(&roadmap, root, &opts);

    assert_eq!(report.violations.len(), 1);
    match &report.violations[0].reason {
        ViolationReason::MissingTestFile(f) => assert_eq!(f, "missing_file.rs"),
        _ => panic!("Wrong violation type"),
    }
}

#[test]
fn test_missing_function_detection() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let file_path = root.join("tests/my_test.rs");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, "fn other_function() {}").unwrap();

    let task = make_task("t2", vec!["tests/my_test.rs::target_function".into()]);
    let roadmap = make_roadmap(vec![task]);
    let opts = AuditOptions { strict: true };

    let report = scan(&roadmap, root, &opts);

    assert_eq!(report.violations.len(), 1);
    match &report.violations[0].reason {
        ViolationReason::MissingTestFunction { file, function } => {
            assert_eq!(file, "tests/my_test.rs");
            assert_eq!(function, "target_function");
        }
        _ => panic!("Wrong violation type"),
    }
}

#[test]
fn test_successful_verification() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let file_path = root.join("tests/valid_test.rs");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, "fn test_my_cool_test() {}").unwrap();

    // Task ID "my-cool-test" matches function "test_my_cool_test"
    let task = make_task(
        "my-cool-test",
        vec!["tests/valid_test.rs::test_my_cool_test".into()],
    );
    let roadmap = make_roadmap(vec![task]);
    let opts = AuditOptions { strict: true };

    let report = scan(&roadmap, root, &opts);

    assert_eq!(report.violations.len(), 0);
    assert_eq!(report.total_checked, 1);
}

#[test]
fn test_naming_convention_mismatch() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let file_path = root.join("tests/naming.rs");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, "fn test_wrong_name() {}").unwrap();

    // Task ID "my-feature" vs "test_wrong_name"
    let task = make_task(
        "my-feature",
        vec!["tests/naming.rs::test_wrong_name".into()],
    );
    let roadmap = make_roadmap(vec![task]);
    let opts = AuditOptions { strict: true };

    let report = scan(&roadmap, root, &opts);

    assert_eq!(report.violations.len(), 1);
    match &report.violations[0].reason {
        ViolationReason::NamingConventionMismatch { expected, actual } => {
            assert_eq!(expected, "test_my_feature");
            assert_eq!(actual, "test_wrong_name");
        }
        _ => panic!("Expected naming mismatch violation"),
    }
}
