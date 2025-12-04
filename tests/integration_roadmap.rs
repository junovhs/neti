// tests/integration_roadmap.rs
use slopchop_core::roadmap::{slugify, CommandBatch, Roadmap};

const SAMPLE: &str = "# Test\n\n## v0.1.0\n\n- [x] **Done**\n- [ ] **Todo**\n";

#[test]
fn test_parse_simple_roadmap() {
    let r = Roadmap::parse(SAMPLE);
    assert_eq!(r.title, "Test");
    assert!(!r.sections.is_empty());
}

#[test]
fn test_parse_extracts_tasks() {
    let r = Roadmap::parse(SAMPLE);
    let t = r.all_tasks();
    assert!(t.len() >= 2);
}

#[test]
fn test_stats_are_correct() {
    let r = Roadmap::parse(SAMPLE);
    let s = r.stats();
    assert_eq!(s.total, s.complete + s.pending);
}

#[test]
fn test_find_task_by_path() {
    let r = Roadmap::parse(SAMPLE);
    let t = r.all_tasks();
    if let Some(first) = t.first() {
        assert!(r.find_task(&first.path).is_some());
    }
}

#[test]
fn test_compact_state_format() {
    let r = Roadmap::parse(SAMPLE);
    let c = r.compact_state();
    assert!(c.contains("Test"));
}

#[test]
fn test_slugify_basic() {
    assert_eq!(slugify("Hello World"), "hello-world");
}

#[test]
fn test_slugify_special_chars() {
    assert_eq!(slugify("hello_world"), "hello-world");
}

#[test]
fn test_slugify_preserves_numbers() {
    assert_eq!(slugify("v0.1.0"), "v0-1-0");
}

#[test]
fn test_parse_extracts_from_larger_text() {
    let input = "text\n===ROADMAP===\nCHECK task\n===END===\nmore";
    let b = CommandBatch::parse(input);
    assert!(!b.commands.is_empty());
}

#[test]
fn test_parse_check_command() {
    let b = CommandBatch::parse("CHECK task-path");
    assert_eq!(b.commands.len(), 1);
}

#[test]
fn test_parse_multiple_commands() {
    let input = "CHECK a\nUNCHECK b\nADD v0.1.0 \"New\"";
    let b = CommandBatch::parse(input);
    assert_eq!(b.commands.len(), 3);
}

#[test]
fn test_parse_add_with_after() {
    let b = CommandBatch::parse("ADD v0.1.0 \"New\" AFTER existing");
    assert_eq!(b.commands.len(), 1);
}

#[test]
fn test_parse_ignores_comments() {
    let input = "# comment\nCHECK a\n// another\nCHECK b";
    let b = CommandBatch::parse(input);
    assert_eq!(b.commands.len(), 2);
}

#[test]
fn test_summary_format() {
    let b = CommandBatch::parse("CHECK a\nCHECK b\nADD v0.1.0 \"x\"");
    let s = b.summary();
    assert!(s.contains("CHECK"));
}
