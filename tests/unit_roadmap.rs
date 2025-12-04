// tests/unit_roadmap.rs
use slopchop_core::roadmap::{CommandBatch, Roadmap};

#[test]
fn test_anchor_extraction() {
    let content = "# P\n\n## v0.1.0\n\n- [x] **F** <!-- test: tests/u.rs::test_f -->\n";
    let r = Roadmap::parse(content);
    let t = r.all_tasks();
    if let Some(task) = t.first() {
        assert!(!task.tests.is_empty());
    }
}

#[test]
fn test_delete_command() {
    let b = CommandBatch::parse("DELETE old-task");
    assert_eq!(b.commands.len(), 1);
}

#[test]
fn test_update_command() {
    let b = CommandBatch::parse("UPDATE task \"New text\"");
    assert_eq!(b.commands.len(), 1);
}

#[test]
fn test_note_command() {
    let b = CommandBatch::parse("NOTE task \"A note\"");
    assert_eq!(b.commands.len(), 1);
}

#[test]
fn test_move_command() {
    let b = CommandBatch::parse("MOVE task-a AFTER task-b");
    assert_eq!(b.commands.len(), 1);
}
