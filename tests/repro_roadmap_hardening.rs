use slopchop_core::roadmap_v2::parser::parse_commands;

#[test]
fn test_roadmap_parser_ignores_inline_markers() {
    // The parser should strictly require ===ROADMAP=== to be on its own line.
    // Inline occurrences should be treated as regular text.
    let input = "This is a comment about ===ROADMAP=== inside a sentence.";
    let cmds = parse_commands(input).unwrap();
    assert!(cmds.is_empty(), "Should not parse inline markers as valid blocks");
}

#[test]
fn test_roadmap_parser_accepts_valid_block() {
    let input = "
===ROADMAP===
CHECK
id = test-task
===ROADMAP===
";
    let cmds = parse_commands(input).unwrap();
    assert_eq!(cmds.len(), 1, "Should parse valid block");
}