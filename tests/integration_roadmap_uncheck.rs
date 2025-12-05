// tests/integration_roadmap_uncheck.rs
//! Tests for UNCHECK command parsing and execution.

use slopchop_core::roadmap::types::{Command, CommandBatch};

#[test]
fn test_parse_uncheck_command() {
    let input = "UNCHECK v0-7-0/test-naming-convention/feature-id-test-function-mapping";
    let batch = CommandBatch::parse(input);

    assert!(batch.errors.is_empty(), "Parse errors: {:?}", batch.errors);
    assert_eq!(batch.commands.len(), 1);

    match &batch.commands[0] {
        Command::Uncheck { path } => {
            assert_eq!(
                path,
                "v0-7-0/test-naming-convention/feature-id-test-function-mapping"
            );
        }
        other => panic!("Expected Uncheck, got {other:?}"),
    }
}

#[test]
fn test_parse_uncheck_in_roadmap_block() {
    let input = r"
Here's my analysis...

===ROADMAP===
UNCHECK parser-hardening/empty-task-id-filtering
===END===

That should do it.
";
    let batch = CommandBatch::parse(input);

    assert!(batch.errors.is_empty(), "Parse errors: {:?}", batch.errors);
    assert_eq!(batch.commands.len(), 1);

    match &batch.commands[0] {
        Command::Uncheck { path } => {
            assert_eq!(path, "parser-hardening/empty-task-id-filtering");
        }
        other => panic!("Expected Uncheck, got {other:?}"),
    }
}

#[test]
fn test_uncheck_with_check_batch() {
    let input = r"
===ROADMAP===
CHECK task-one
UNCHECK task-two
CHECK task-three
===END===
";
    let batch = CommandBatch::parse(input);

    assert!(batch.errors.is_empty());
    assert_eq!(batch.commands.len(), 3);

    assert!(matches!(&batch.commands[0], Command::Check { path } if path == "task-one"));
    assert!(matches!(&batch.commands[1], Command::Uncheck { path } if path == "task-two"));
    assert!(matches!(&batch.commands[2], Command::Check { path } if path == "task-three"));
}

#[test]
fn test_uncheck_summary_included() {
    let input = "UNCHECK some-task";
    let batch = CommandBatch::parse(input);

    let summary = batch.summary();
    assert!(summary.contains("UNCHECK"), "Summary: {summary}");
}