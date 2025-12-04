use slopchop_core::roadmap::Roadmap;

#[test]
fn test_empty_id_skipped() {
    let content = r"
# Test Roadmap
## Section
- [ ] Valid Task
- [ ] !!!
- [ ] ???
";
    let roadmap = Roadmap::parse(content);
    let tasks = roadmap.all_tasks();

    // Should only contain "valid-task"
    // "!!!" and "???" slugify to empty strings and should be filtered out
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, "valid-task");
}

#[test]
fn test_id_collision_resolved() {
    let content = r"
# Test Roadmap
## Section
- [ ] Duplicate
- [ ] Duplicate
- [ ] Duplicate
";
    let roadmap = Roadmap::parse(content);
    let tasks = roadmap.all_tasks();

    assert_eq!(tasks.len(), 3);

    // First one gets the base slug
    assert_eq!(tasks[0].id, "duplicate");
    // Subsequent ones get numeric suffixes
    assert_eq!(tasks[1].id, "duplicate-1");
    assert_eq!(tasks[2].id, "duplicate-2");
}

#[test]
fn test_anchor_id_extraction() {
    let content = r"
# Test Roadmap
## Section
- [ ] Task with Anchor <!-- test: tests/foo.rs::my_function -->
- [ ] Task without Anchor
";
    let roadmap = Roadmap::parse(content);
    let tasks = roadmap.all_tasks();

    // First task should derive ID from the function name in the anchor
    assert_eq!(tasks[0].id, "my-function");

    // Second task derives ID from text
    assert_eq!(tasks[1].id, "task-without-anchor");

    // Verify the test path was parsed correctly too
    assert_eq!(tasks[0].tests[0], "tests/foo.rs::my_function");
}
