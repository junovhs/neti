//! Unit tests for roadmap diff engine.
//!
//! These tests verify the "Wicked Smart" diff algorithm correctly infers
//! user intent when comparing two roadmap versions, generating atomic
//! commands rather than destructive rewrites.

use warden_core::roadmap::{diff, Roadmap};

/// Verifies that when a task's text changes but the test anchor remains,
/// the diff engine generates an UPDATE command instead of DELETE + ADD.
///
/// This is the core "anchor-based matching" behavior that enables
/// accurate intent detection.
#[test]
fn test_text_change_is_update() {
    let current = r#"# Roadmap

## Features

- [x] **Old descriptive text** <!-- test: tests/feature.rs::test_my_feature -->
"#;

    let incoming = r#"# Roadmap

## Features

- [x] **New improved text** <!-- test: tests/feature.rs::test_my_feature -->
"#;

    let curr_roadmap = Roadmap::parse(current);
    let inc_roadmap = Roadmap::parse(incoming);

    let commands = diff::diff(&curr_roadmap, &inc_roadmap);

    // Should have exactly one UPDATE, not DELETE + ADD
    let updates: Vec<_> = commands
        .iter()
        .filter(|c| matches!(c, warden_core::roadmap::Command::Update { .. }))
        .collect();

    let deletes: Vec<_> = commands
        .iter()
        .filter(|c| matches!(c, warden_core::roadmap::Command::Delete { .. }))
        .collect();

    let adds: Vec<_> = commands
        .iter()
        .filter(|c| matches!(c, warden_core::roadmap::Command::Add { .. }))
        .collect();

    assert_eq!(
        updates.len(),
        1,
        "Expected 1 UPDATE command, got {}: {:?}",
        updates.len(),
        commands
    );
    assert_eq!(
        deletes.len(),
        0,
        "Expected 0 DELETE commands, got {}: {:?}",
        deletes.len(),
        commands
    );
    assert_eq!(
        adds.len(),
        0,
        "Expected 0 ADD commands, got {}: {:?}",
        adds.len(),
        commands
    );

    // Verify the UPDATE has correct content
    if let warden_core::roadmap::Command::Update { path, text } = &updates[0] {
        assert!(
            path.contains("test-my-feature"),
            "UPDATE path should reference test-my-feature, got: {}",
            path
        );
        assert_eq!(
            text, "New improved text",
            "UPDATE text should be the new text"
        );
    }
}

/// Verifies that status changes are detected as CHECK/UNCHECK commands.
#[test]
fn test_status_change_detected() {
    let current = r#"# Roadmap

## Features

- [ ] **Pending task** <!-- test: tests/feature.rs::test_pending -->
- [x] **Complete task** <!-- test: tests/feature.rs::test_complete -->
"#;

    let incoming = r#"# Roadmap

## Features

- [x] **Pending task** <!-- test: tests/feature.rs::test_pending -->
- [ ] **Complete task** <!-- test: tests/feature.rs::test_complete -->
"#;

    let curr_roadmap = Roadmap::parse(current);
    let inc_roadmap = Roadmap::parse(incoming);

    let commands = diff::diff(&curr_roadmap, &inc_roadmap);

    let checks: Vec<_> = commands
        .iter()
        .filter(|c| matches!(c, warden_core::roadmap::Command::Check { .. }))
        .collect();

    let unchecks: Vec<_> = commands
        .iter()
        .filter(|c| matches!(c, warden_core::roadmap::Command::Uncheck { .. }))
        .collect();

    assert_eq!(checks.len(), 1, "Expected 1 CHECK command");
    assert_eq!(unchecks.len(), 1, "Expected 1 UNCHECK command");
}

/// Verifies that deleted tasks generate DELETE commands.
#[test]
fn test_deleted_task_detected() {
    let current = r#"# Roadmap

## Features

- [x] **Keep this** <!-- test: tests/a.rs::test_keep -->
- [x] **Delete this** <!-- test: tests/b.rs::test_delete -->
"#;

    let incoming = r#"# Roadmap

## Features

- [x] **Keep this** <!-- test: tests/a.rs::test_keep -->
"#;

    let curr_roadmap = Roadmap::parse(current);
    let inc_roadmap = Roadmap::parse(incoming);

    let commands = diff::diff(&curr_roadmap, &inc_roadmap);

    let deletes: Vec<_> = commands
        .iter()
        .filter_map(|c| {
            if let warden_core::roadmap::Command::Delete { path } = c {
                Some(path.as_str())
            } else {
                None
            }
        })
        .collect();

    assert_eq!(deletes.len(), 1, "Expected 1 DELETE command");
    assert!(
        deletes[0].contains("test-delete"),
        "DELETE should target test-delete, got: {}",
        deletes[0]
    );
}

/// Verifies that new tasks generate ADD commands.
#[test]
fn test_new_task_detected() {
    let current = r#"# Roadmap

## Features

- [x] **Existing** <!-- test: tests/a.rs::test_existing -->
"#;

    let incoming = r#"# Roadmap

## Features

- [x] **Existing** <!-- test: tests/a.rs::test_existing -->
- [ ] **Brand new task** <!-- test: tests/b.rs::test_new -->
"#;

    let curr_roadmap = Roadmap::parse(current);
    let inc_roadmap = Roadmap::parse(incoming);

    let commands = diff::diff(&curr_roadmap, &inc_roadmap);

    let adds: Vec<_> = commands
        .iter()
        .filter(|c| matches!(c, warden_core::roadmap::Command::Add { .. }))
        .collect();

    assert_eq!(adds.len(), 1, "Expected 1 ADD command");
}

/// Verifies that tasks moved between sections generate MOVE commands.
#[test]
fn test_moved_task_detected() {
    let current = r#"# Roadmap

## Section A

- [x] **Mobile task** <!-- test: tests/x.rs::test_mobile -->

## Section B

"#;

    let incoming = r#"# Roadmap

## Section A

## Section B

- [x] **Mobile task** <!-- test: tests/x.rs::test_mobile -->
"#;

    let curr_roadmap = Roadmap::parse(current);
    let inc_roadmap = Roadmap::parse(incoming);

    let commands = diff::diff(&curr_roadmap, &inc_roadmap);

    let moves: Vec<_> = commands
        .iter()
        .filter(|c| matches!(c, warden_core::roadmap::Command::Move { .. }))
        .collect();

    assert_eq!(moves.len(), 1, "Expected 1 MOVE command");
}

/// Verifies that new sections generate SECTION commands.
#[test]
fn test_new_section_detected() {
    let current = r#"# Roadmap

## Existing Section

- [x] **Task** <!-- test: tests/a.rs::test_a -->
"#;

    let incoming = r#"# Roadmap

## Existing Section

- [x] **Task** <!-- test: tests/a.rs::test_a -->

## Brand New Section

- [ ] **New task**
"#;

    let curr_roadmap = Roadmap::parse(current);
    let inc_roadmap = Roadmap::parse(incoming);

    let commands = diff::diff(&curr_roadmap, &inc_roadmap);

    let sections: Vec<_> = commands
        .iter()
        .filter(|c| matches!(c, warden_core::roadmap::Command::AddSection { .. }))
        .collect();

    assert_eq!(sections.len(), 1, "Expected 1 SECTION command");
}

/// Verifies that DELETE commands never have empty paths.
/// This was a critical bug that caused invalid command generation.
#[test]
fn test_no_empty_delete_paths() {
    let current = r#"# Roadmap

## Section

- [x] **Valid task** <!-- test: tests/a.rs::test_valid -->
- [x] **Another valid** <!-- test: tests/b.rs::test_another -->
"#;

    let incoming = r#"# Roadmap

## Section

"#;

    let curr_roadmap = Roadmap::parse(current);
    let inc_roadmap = Roadmap::parse(incoming);

    let commands = diff::diff(&curr_roadmap, &inc_roadmap);

    for cmd in &commands {
        if let warden_core::roadmap::Command::Delete { path } = cmd {
            assert!(
                !path.is_empty(),
                "DELETE command has empty path: {:?}",
                cmd
            );
        }
    }
}

/// Verifies combined operations: text change + status change on same task.
#[test]
fn test_combined_update_and_status_change() {
    let current = r#"# Roadmap

## Features

- [ ] **Old text** <!-- test: tests/x.rs::test_combo -->
"#;

    let incoming = r#"# Roadmap

## Features

- [x] **New text** <!-- test: tests/x.rs::test_combo -->
"#;

    let curr_roadmap = Roadmap::parse(current);
    let inc_roadmap = Roadmap::parse(incoming);

    let commands = diff::diff(&curr_roadmap, &inc_roadmap);

    let has_check = commands
        .iter()
        .any(|c| matches!(c, warden_core::roadmap::Command::Check { .. }));

    let has_update = commands
        .iter()
        .any(|c| matches!(c, warden_core::roadmap::Command::Update { .. }));

    assert!(has_check, "Expected CHECK command for status change");
    assert!(has_update, "Expected UPDATE command for text change");
}

/// Verifies no commands generated when roadmaps are identical.
#[test]
fn test_identical_roadmaps_no_commands() {
    let content = r#"# Roadmap

## Features

- [x] **Task one** <!-- test: tests/a.rs::test_one -->
- [ ] **Task two** <!-- test: tests/b.rs::test_two -->
"#;

    let curr_roadmap = Roadmap::parse(content);
    let inc_roadmap = Roadmap::parse(content);

    let commands = diff::diff(&curr_roadmap, &inc_roadmap);

    assert!(
        commands.is_empty(),
        "Identical roadmaps should produce no commands, got: {:?}",
        commands
    );
}