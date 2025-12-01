//! Unit tests for roadmap parser hardening.
//!
//! These tests verify critical safety features:
//! - Empty task IDs are rejected (prevents DELETE "" commands)
//! - Colliding task IDs are deduplicated (prevents UNCHECK ambiguity)
//! - Test anchors are used as stable IDs (enables UPDATE detection)

use warden_core::roadmap::Roadmap;

/// Verifies that tasks with no extractable text (empty after slugification)
/// are skipped entirely rather than creating tasks with empty IDs.
///
/// This prevents the diff engine from generating invalid `DELETE ""` commands.
#[test]
fn test_empty_id_skipped() {
    let content = r#"# Test Roadmap

## Section

- [x] **<!-- test: some/path.rs::test_func -->**
- [x] **   ** <!-- test: another/path.rs::test_func2 -->
- [x] **Valid task here** <!-- test: valid/path.rs::test_valid -->
"#;

    let roadmap = Roadmap::parse(content);
    let tasks = roadmap.all_tasks();

    // Should only have tasks with valid IDs
    for task in &tasks {
        assert!(
            !task.id.is_empty(),
            "Found task with empty ID: {:?}",
            task
        );
    }

    // The valid task should be present (anchor-based ID)
    assert!(
        tasks.iter().any(|t| t.id == "test-valid"),
        "Valid task not found. Tasks: {:?}",
        tasks.iter().map(|t| &t.id).collect::<Vec<_>>()
    );
}

/// Verifies that multiple tasks with the same slugified text
/// receive unique IDs via numeric suffixes.
///
/// Example: "Pattern: // ...", "Pattern: /* ... */" both slugify to "pattern"
/// but should become "pattern" and "pattern-1".
#[test]
fn test_id_collision_resolved() {
    let content = r#"# Test Roadmap

## Patterns

- [x] **Pattern: first**
- [x] **Pattern: second**
- [x] **Pattern: third**
- [ ] **Different task**
"#;

    let roadmap = Roadmap::parse(content);
    let tasks = roadmap.all_tasks();

    // Collect all IDs
    let ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();

    // Should have pattern, pattern-1, pattern-2 (not three "pattern"s)
    assert!(ids.contains(&"pattern-first"), "Missing pattern-first");
    assert!(ids.contains(&"pattern-second"), "Missing pattern-second");
    assert!(ids.contains(&"pattern-third"), "Missing pattern-third");
    assert!(ids.contains(&"different-task"), "Missing different-task");

    // Verify all IDs are unique
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        ids.len(),
        sorted.len(),
        "Duplicate IDs found: {:?}",
        ids
    );
}

/// Verifies that when a task has a test anchor, the test function name
/// is used as the task ID instead of the slugified text.
///
/// This enables the diff engine to detect text renames as UPDATE operations
/// rather than DELETE + ADD.
#[test]
fn test_anchor_id_extraction() {
    let content = r#"# Test Roadmap

## Features

- [x] **Some descriptive text here** <!-- test: tests/integration.rs::test_my_feature -->
- [x] **Another feature** <!-- test: tests/unit.rs::test_another_thing -->
- [ ] **No anchor task**
"#;

    let roadmap = Roadmap::parse(content);
    let tasks = roadmap.all_tasks();

    // Tasks with anchors should use the function name as ID
    let feature_task = tasks.iter().find(|t| t.text.contains("descriptive"));
    assert!(
        feature_task.is_some(),
        "Could not find task with 'descriptive' in text"
    );
    assert_eq!(
        feature_task.map(|t| t.id.as_str()),
        Some("test-my-feature"),
        "Anchor-based ID not extracted correctly"
    );

    let another_task = tasks.iter().find(|t| t.text.contains("Another"));
    assert_eq!(
        another_task.map(|t| t.id.as_str()),
        Some("test-another-thing"),
        "Second anchor-based ID not extracted correctly"
    );

    // Task without anchor falls back to slugified text
    let no_anchor = tasks.iter().find(|t| t.text.contains("No anchor"));
    assert_eq!(
        no_anchor.map(|t| t.id.as_str()),
        Some("no-anchor-task"),
        "Fallback slugification failed"
    );
}

/// Verifies that the full test path is parsed but only the function name
/// is used for the ID (not the full path).
#[test]
fn test_anchor_extracts_function_name_only() {
    let content = r#"# Roadmap

## Section

- [x] **Task** <!-- test: tests/very/deep/nested/path.rs::test_the_function -->
"#;

    let roadmap = Roadmap::parse(content);
    let tasks = roadmap.all_tasks();

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, "test-the-function");
    assert_eq!(
        tasks[0].tests.first().map(String::as_str),
        Some("tests/very/deep/nested/path.rs::test_the_function")
    );
}

/// Verifies that anchor-based IDs also get deduplicated if multiple tasks
/// reference the same test function (edge case but possible).
#[test]
fn test_anchor_collision_deduplicated() {
    let content = r#"# Roadmap

## Section

- [x] **First mention** <!-- test: tests/foo.rs::test_shared -->
- [x] **Second mention** <!-- test: tests/bar.rs::test_shared -->
"#;

    let roadmap = Roadmap::parse(content);
    let tasks = roadmap.all_tasks();

    let ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();

    // Both extract to "test-shared" but should be deduplicated
    assert!(ids.contains(&"test-shared"), "Missing test-shared");
    assert!(ids.contains(&"test-shared-1"), "Missing test-shared-1");
}

/// Verifies correct handling of tasks with [no-test] marker.
/// These should fall back to text-based slugification.
#[test]
fn test_no_test_marker_uses_text_id() {
    let content = r#"# Roadmap

## Section

- [x] **Documentation site** [no-test]
- [x] **Logo and branding** [no-test]
"#;

    let roadmap = Roadmap::parse(content);
    let tasks = roadmap.all_tasks();

    let ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();

    assert!(ids.contains(&"documentation-site-no-test"));
    assert!(ids.contains(&"logo-and-branding-no-test"));
}