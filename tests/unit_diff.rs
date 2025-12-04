use slopchop_core::roadmap::{diff::diff, Command, Roadmap};

#[test]
fn test_text_change_is_update() {
    // Current state: Task has specific text and an anchor
    let current_content = r"
# Roadmap
## Section
- [ ] Original Text <!-- test: tests/check.rs::stable_id -->
";

    // Incoming state: Text changed, but anchor (and thus ID) remains same
    let incoming_content = r"
# Roadmap
## Section
- [ ] Updated Text <!-- test: tests/check.rs::stable_id -->
";

    let current = Roadmap::parse(current_content);
    let incoming = Roadmap::parse(incoming_content);

    let commands = diff(&current, &incoming);

    assert_eq!(commands.len(), 1);

    match &commands[0] {
        Command::Update { path, text } => {
            // ID should be 'stable-id' derived from anchor
            assert_eq!(path, "stable-id");
            assert_eq!(text, "Updated Text");
        }
        _ => panic!("Expected UPDATE command, got {:?}", commands[0]),
    }
}
