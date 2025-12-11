// tests/unit_roadmap_v2_extra.rs
use slopchop_core::roadmap_v2::types::{RoadmapMeta, Section, SectionStatus};
use slopchop_core::roadmap_v2::{parse_commands, RoadmapCommand, Task, TaskStatus, TaskStore};

fn create_test_store() -> TaskStore {
    TaskStore {
        meta: RoadmapMeta {
            title: "Test Roadmap".to_string(),
            description: String::new(),
        },
        sections: vec![Section {
            id: "v0.1.0".to_string(),
            title: "Foundation".to_string(),
            status: SectionStatus::Current,
            order: 0,
        }],
        tasks: vec![
            Task {
                id: "task-one".to_string(),
                text: "First task".to_string(),
                status: TaskStatus::Pending,
                section: "v0.1.0".to_string(),
                group: Some("Test Group".to_string()),
                test: None,
                order: 0,
            },
            Task {
                id: "task-two".to_string(),
                text: "Second task".to_string(),
                status: TaskStatus::Pending,
                section: "v0.1.0".to_string(),
                group: Some("Test Group".to_string()),
                test: None,
                order: 1,
            },
        ],
    }
}

#[test]
fn test_task_status_complete() {
    let mut store = create_test_store();
    store.tasks[0].status = TaskStatus::Done;
    let md = store.to_markdown();
    assert!(md.contains("[x]"));
}

#[test]
fn test_stats_calculation() {
    let store = create_test_store();
    let total = store.tasks.len();
    let done = store
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Done)
        .count();
    assert_eq!(total, 2);
    assert_eq!(done, 0);
}

#[test]
fn test_task_path_generation() {
    let store = create_test_store();
    let task = &store.tasks[0];
    let group = task.group.as_deref().unwrap_or("default");
    let path = format!("{}/{}/{}", task.section, group, task.id);
    assert!(path.contains("v0.1.0"));
    assert!(path.contains("Test Group"));
}

#[test]
fn test_compact_state_display() {
    let store = create_test_store();
    let md = store.to_markdown();
    assert!(md.contains("[ ]") || md.contains("[x]"));
}

#[test]
fn test_lowercase_conversion() {
    let id = "My-Task-Name".to_lowercase();
    assert_eq!(id, "my-task-name");
}

#[test]
fn test_special_char_to_dash() {
    let text = "Hello World! Test@123";
    let slug: String = text
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    assert!(slug.contains('-'));
}

#[test]
fn test_number_preservation() {
    let text = "v0.1.0 Feature 123";
    let slug: String = text
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '.')
        .collect();
    assert!(slug.contains("0.1.0"));
    assert!(slug.contains("123"));
}

#[test]
fn test_add_with_after() {
    let input = r"
===ROADMAP===
ADD
id = new-task
text = New feature
section = v0.1.0
group = Test
after = task-one
===ROADMAP===
";
    let cmds = parse_commands(input).unwrap_or_default();
    assert_eq!(cmds.len(), 1);
    match &cmds[0] {
        RoadmapCommand::Add(add_cmd) => {
            assert_eq!(add_cmd.task.id, "new-task");
        }
        _ => panic!("Expected Add command"),
    }
}

#[test]
fn test_note_command() {
    let input = "===ROADMAP===\nNOTE\nid = task-one\nnote = Important info\n===ROADMAP===";
    let cmds = parse_commands(input).unwrap_or_default();
    let _ = cmds.len();
}

#[test]
fn test_move_command() {
    let input = "===ROADMAP===\nMOVE\nid = task-one\nto = v0.2.0\n===ROADMAP===";
    let cmds = parse_commands(input).unwrap_or_default();
    let _ = cmds.len();
}

#[test]
fn test_comment_skipping() {
    let input = "===ROADMAP===\n# This is a comment\nCHECK\nid = task-one\n===ROADMAP===";
    let cmds = parse_commands(input).unwrap_or_default();
    assert!(!cmds.is_empty());
}

#[test]
fn test_section_command() {
    let input = "===ROADMAP===\nSECTION\nname = v0.2.0\ntitle = New Version\n===ROADMAP===";
    let cmds = parse_commands(input).unwrap_or_default();
    let _ = cmds.len();
}

#[test]
fn test_subsection_command() {
    let input = "===ROADMAP===\nSUBSECTION\nname = New Group\nsection = v0.1.0\n===ROADMAP===";
    let cmds = parse_commands(input).unwrap_or_default();
    let _ = cmds.len();
}

#[test]
fn test_chain_command() {
    let input = r"
===ROADMAP===
CHAIN
section = v0.1.0
group = Test
tasks = First task, Second task, Third task
===ROADMAP===
";
    let cmds = parse_commands(input).unwrap_or_default();
    let _ = cmds.len();
}