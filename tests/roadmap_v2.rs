// tests/roadmap_v2.rs
use slopchop_core::roadmap_v2::types::{AddCommand, AfterTarget};
use slopchop_core::roadmap_v2::{parse_commands, RoadmapCommand, Task, TaskStatus, TaskStore};
use tempfile::TempDir;

#[test]
fn test_store_load_save() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let path = dir.path().join("tasks.toml");

    let mut store = TaskStore::default();
    store.meta.title = "Test Roadmap".to_string();
    store.sections.push(slopchop_core::roadmap_v2::types::Section {
        id: "v1".to_string(),
        title: "v1.0.0".to_string(),
        status: slopchop_core::roadmap_v2::types::SectionStatus::Current,
        order: 0,
    });

    store.save(Some(&path)).expect("Save failed");
    assert!(path.exists());

    let loaded = TaskStore::load(Some(&path)).expect("Load failed");
    assert_eq!(loaded.meta.title, "Test Roadmap");
    assert_eq!(loaded.sections.len(), 1);
}

#[test]
fn test_store_apply_check() {
    let mut store = create_test_store();

    let cmd = RoadmapCommand::Check {
        id: "task-1".to_string(),
    };
    store.apply(cmd).expect("Apply failed");

    let task = store.tasks.iter().find(|t| t.id == "task-1").unwrap();
    assert_eq!(task.status, TaskStatus::Done);
}

#[test]
fn test_store_apply_uncheck() {
    let mut store = create_test_store();

    store
        .apply(RoadmapCommand::Check {
            id: "task-1".to_string(),
        })
        .expect("Check failed");

    store
        .apply(RoadmapCommand::Uncheck {
            id: "task-1".to_string(),
        })
        .expect("Uncheck failed");

    let task = store.tasks.iter().find(|t| t.id == "task-1").unwrap();
    assert_eq!(task.status, TaskStatus::Pending);
}

#[test]
fn test_store_apply_add() {
    let mut store = create_test_store();

    let new_task = Task {
        id: "new-task".to_string(),
        text: "New Feature".to_string(),
        status: TaskStatus::Pending,
        section: "v1".to_string(),
        group: None,
        test: Some("tests/unit.rs::test_new".to_string()),
        order: 10,
    };

    let add_cmd = AddCommand {
        task: new_task,
        after: AfterTarget::End,
    };

    store
        .apply(RoadmapCommand::Add(add_cmd))
        .expect("Add failed");

    assert_eq!(store.tasks.len(), 2);
    let added = store.tasks.iter().find(|t| t.id == "new-task").unwrap();
    assert_eq!(added.text, "New Feature");
}

#[test]
fn test_store_apply_delete() {
    let mut store = create_test_store();

    store
        .apply(RoadmapCommand::Delete {
            id: "task-1".to_string(),
        })
        .expect("Delete failed");

    assert!(store.tasks.is_empty());
}

#[test]
fn test_store_apply_update() {
    let mut store = create_test_store();

    store
        .apply(RoadmapCommand::Update {
            id: "task-1".to_string(),
            fields: slopchop_core::roadmap_v2::types::TaskUpdate {
                text: Some("Updated Text".to_string()),
                test: Some("tests/new.rs::test_fn".to_string()),
                section: None,
                group: None,
            },
        })
        .expect("Update failed");

    let task = store.tasks.iter().find(|t| t.id == "task-1").unwrap();
    assert_eq!(task.text, "Updated Text");
    assert_eq!(task.test, Some("tests/new.rs::test_fn".to_string()));
}

#[test]
fn test_parse_check_command() {
    let input = "===ROADMAP===\nCHECK\nid = my-task\n===ROADMAP===";
    let cmds = parse_commands(input).expect("Parse failed");

    assert_eq!(cmds.len(), 1);
    match &cmds[0] {
        RoadmapCommand::Check { id } => assert_eq!(id, "my-task"),
        _ => panic!("Expected Check command"),
    }
}

#[test]
fn test_parse_uncheck_command() {
    let input = "===ROADMAP===\nUNCHECK\nid = task-abc\n===ROADMAP===";
    let cmds = parse_commands(input).expect("Parse failed");

    assert_eq!(cmds.len(), 1);
    match &cmds[0] {
        RoadmapCommand::Uncheck { id } => assert_eq!(id, "task-abc"),
        _ => panic!("Expected Uncheck command"),
    }
}

#[test]
fn test_parse_add_command() {
    let input = r"===ROADMAP===
ADD
id = new-feature
text = Support Go Language
section = v0.8.0
group = Lang Support
test = tests/unit.rs::test_go
===ROADMAP===";

    let cmds = parse_commands(input).expect("Parse failed");

    assert_eq!(cmds.len(), 1);
    match &cmds[0] {
        RoadmapCommand::Add(add_cmd) => {
            assert_eq!(add_cmd.task.id, "new-feature");
            assert_eq!(add_cmd.task.text, "Support Go Language");
            assert_eq!(add_cmd.task.section, "v0.8.0");
            assert_eq!(add_cmd.task.group, Some("Lang Support".to_string()));
            assert_eq!(add_cmd.task.test, Some("tests/unit.rs::test_go".to_string()));
        }
        _ => panic!("Expected Add command"),
    }
}

#[test]
fn test_parse_multiple_commands() {
    let input = r"
===ROADMAP===
CHECK
id = task-1
===ROADMAP===

Some text in between

===ROADMAP===
CHECK
id = task-2
===ROADMAP===
";

    let cmds = parse_commands(input).expect("Parse failed");
    assert_eq!(cmds.len(), 2);
}

#[test]
fn test_generator_markdown() {
    let store = create_test_store();
    let md = store.to_markdown();

    assert!(md.contains("# Test Roadmap"));
    assert!(md.contains("## v1.0.0"));
    assert!(md.contains("- [ ] **Task One**"));
}

#[test]
fn test_generator_with_done_task() {
    let mut store = create_test_store();
    store.tasks[0].status = TaskStatus::Done;

    let md = store.to_markdown();
    assert!(md.contains("- [x] **Task One**"));
}

#[test]
fn test_generator_with_test_anchor() {
    let mut store = create_test_store();
    store.tasks[0].test = Some("tests/unit.rs::test_fn".to_string());

    let md = store.to_markdown();
    assert!(md.contains("<!-- test: tests/unit.rs::test_fn -->"));
}

fn create_test_store() -> TaskStore {
    use slopchop_core::roadmap_v2::types::{RoadmapMeta, Section, SectionStatus};

    TaskStore {
        meta: RoadmapMeta {
            title: "Test Roadmap".to_string(),
            description: String::new(),
        },
        sections: vec![Section {
            id: "v1".to_string(),
            title: "v1.0.0".to_string(),
            status: SectionStatus::Current,
            order: 0,
        }],
        tasks: vec![Task {
            id: "task-1".to_string(),
            text: "Task One".to_string(),
            status: TaskStatus::Pending,
            section: "v1".to_string(),
            group: None,
            test: None,
            order: 0,
        }],
    }
}