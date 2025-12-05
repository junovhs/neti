// tests/unit_roadmap_cmd.rs
//! Tests for roadmap command parsing and execution.

use slopchop_core::roadmap::cmd_runner::apply_commands;
use slopchop_core::roadmap::types::{Command, CommandBatch, Roadmap};

#[test]
fn test_section_command() {
    let input = r#"SECTION "v0.11.0 - New Features""#;
    let batch = CommandBatch::parse(input);
    assert!(batch.errors.is_empty());
    assert_eq!(batch.commands.len(), 1);
    assert!(matches!(&batch.commands[0], Command::AddSection { heading } if heading == "v0.11.0 - New Features"));
}

#[test]
fn test_section_command_unquoted() {
    let input = "SECTION v0.12.0 - Performance";
    let batch = CommandBatch::parse(input);
    assert!(batch.errors.is_empty());
    assert!(matches!(&batch.commands[0], Command::AddSection { heading } if heading == "v0.12.0 - Performance"));
}

#[test]
fn test_section_execution() {
    let mut roadmap = Roadmap::parse("# Test\n\n## v0.1.0\n\n- [x] **Task**\n");
    let batch = CommandBatch::parse(r#"SECTION "v0.2.0 - New""#);
    let results = apply_commands(&mut roadmap, &batch);
    assert_eq!(results.len(), 1);
    assert!(roadmap.raw.contains("## v0.2.0 - New"));
}

#[test]
fn test_section_in_batch() {
    let input = "===ROADMAP===\nSECTION v0.99.0\nADD v0.99.0 \"Feature\"\n===END===";
    let batch = CommandBatch::parse(input);
    assert!(batch.errors.is_empty());
    assert_eq!(batch.commands.len(), 2);
}

#[test]
fn test_subsection_command() {
    let input = r#"SUBSECTION v0.7.0 "New Subsection""#;
    let batch = CommandBatch::parse(input);
    assert!(batch.errors.is_empty());
    match &batch.commands[0] {
        Command::AddSubsection { parent, heading } => {
            assert_eq!(parent, "v0.7.0");
            assert_eq!(heading, "New Subsection");
        }
        other => panic!("Expected AddSubsection, got {other:?}"),
    }
}

#[test]
fn test_subsection_execution() {
    let mut roadmap = Roadmap::parse("# Test\n\n## v0.7.0\n\n- [x] **Task**\n\n## v0.8.0\n");
    let batch = CommandBatch::parse(r#"SUBSECTION v0.7.0 "Parser""#);
    let results = apply_commands(&mut roadmap, &batch);
    assert_eq!(results.len(), 1);
    assert!(roadmap.raw.contains("### Parser"));
}

#[test]
fn test_chain_command() {
    let input = r#"CHAIN v0.7.0 "First task" "Second task" "Third task""#;
    let batch = CommandBatch::parse(input);
    assert!(batch.errors.is_empty(), "Errors: {:?}", batch.errors);
    match &batch.commands[0] {
        Command::Chain { parent, items } => {
            assert_eq!(parent, "v0.7.0");
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], "First task");
            assert_eq!(items[1], "Second task");
            assert_eq!(items[2], "Third task");
        }
        other => panic!("Expected Chain, got {other:?}"),
    }
}

#[test]
fn test_chain_execution() {
    let mut roadmap = Roadmap::parse("# Test\n\n## v0.7.0 - Current\n\n- [x] **Existing**\n");
    let batch = CommandBatch::parse(r#"CHAIN v0.7.0 "Task A" "Task B""#);
    let results = apply_commands(&mut roadmap, &batch);
    assert_eq!(results.len(), 2);
    assert!(roadmap.raw.contains("Task A"));
    assert!(roadmap.raw.contains("Task B"));
}