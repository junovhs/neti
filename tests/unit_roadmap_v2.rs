// tests/unit_roadmap_v2.rs
use slopchop_core::roadmap_v2::types::{AddCommand, AfterTarget, RoadmapMeta, Section, SectionStatus};
use slopchop_core::roadmap_v2::{parse_commands, RoadmapCommand, Task, TaskStatus, TaskStore};

#[test]
fn test_store_check_command() {
    let mut store = create_test_store();
    let cmds = parse_commands("===ROADMAP===\nCHECK\nid = task-one\n===ROADMAP===").unwrap_or_default();
    assert_eq!(cmds.len(), 1);
    for cmd in cmds {
        store.apply(cmd).ok();
    }
    let task = store.tasks.iter().find(|t| t.id == "task-one");
    assert!(task.is_some_and(|t| t.status == TaskStatus::Done));
}

#[test]
fn test_store_uncheck_command() {
    let mut store = create_test_store();
    store.tasks[0].status = TaskStatus::Done;
    let cmds = parse_commands("===ROADMAP===\nUNCHECK\nid = task-one\n===ROADMAP===").unwrap_or_default();
    for cmd in cmds {
        store.apply(cmd).ok();
    }
    let task = store.tasks.iter().find(|t| t.id == "task-one");
    assert!(task.is_some_and(|t| t.status == TaskStatus::Pending));
}

#[test]
fn test_store_add_command() {
    let mut store = create_test_store();
    let input = "===ROADMAP===\nADD\nid = new-task\ntext = A brand new feature\nsection = v0.1.0\ngroup = New Group\ntest = tests/new.rs::test_new\n===ROADMAP===";
    let cmds = parse_commands(input).unwrap_or_default();
    for cmd in cmds {
        store.apply(cmd).ok();
    }
    assert_eq!(store.tasks.len(), 3);
    let task = store.tasks.iter().find(|t| t.id == "new-task");
    assert!(task.is_some());
    assert_eq!(task.map(|t| t.text.as_str()), Some("A brand new feature"));
}

#[test]
fn test_store_delete_command() {
    let mut store = create_test_store();
    assert_eq!(store.tasks.len(), 2);
    let cmds = parse_commands("===ROADMAP===\nDELETE\nid = task-two\n===ROADMAP===").unwrap_or_default();
    for cmd in cmds {
        store.apply(cmd).ok();
    }
    assert_eq!(store.tasks.len(), 1);
    assert!(store.tasks.iter().all(|t| t.id != "task-two"));
}

#[test]
fn test_store_update_command() {
    let mut store = create_test_store();
    let input = "===ROADMAP===\nUPDATE\nid = task-one\ntext = Updated task text\ntest = tests/updated.rs::test_updated\n===ROADMAP===";
    let cmds = parse_commands(input).unwrap_or_default();
    for cmd in cmds {
        store.apply(cmd).ok();
    }
    let task = store.tasks.iter().find(|t| t.id == "task-one");
    assert_eq!(task.map(|t| t.text.as_str()), Some("Updated task text"));
    assert_eq!(task.and_then(|t| t.test.as_deref()), Some("tests/updated.rs::test_updated"));
}

#[test]
fn test_generator_basic_markdown() {
    let store = create_test_store();
    let md = store.to_markdown();
    assert!(md.contains("# Test Roadmap"));
    assert!(md.contains("## v0.1.0 - Foundation ?"));
    assert!(md.contains("### Test Group"));
    assert!(md.contains("- [ ] **First task**"));
    assert!(md.contains("- [ ] **Second task**"));
}

#[test]
fn test_generator_includes_test_anchors() {
    let mut store = create_test_store();
    store.tasks[0].test = Some("tests/unit.rs::test_fn".to_string());
    store.tasks[0].status = TaskStatus::Done;
    let md = store.to_markdown();
    assert!(md.contains("[x] **First task** <!-- test: tests/unit.rs::test_fn -->"));
}

#[test]
fn test_generator_notest_marker() {
    let mut store = create_test_store();
    store.tasks[0].status = TaskStatus::NoTest;
    let md = store.to_markdown();
    assert!(md.contains("[x] **First task** [no-test]"));
}

#[test]
fn test_duplicate_add_rejected() {
    let mut store = create_test_store();
    let cmd = RoadmapCommand::Add(AddCommand {
        task: Task {
            id: "task-one".to_string(),
            text: "Duplicate".to_string(),
            status: TaskStatus::Pending,
            section: "v0.1.0".to_string(),
            group: None,
            test: None,
            order: 0,
        },
        after: AfterTarget::End,
    });
    let result = store.apply(cmd);
    assert!(result.is_err());
}

#[test]
fn test_missing_task_rejected() {
    let mut store = create_test_store();
    let cmds = parse_commands("===ROADMAP===\nCHECK\nid = nonexistent\n===ROADMAP===").unwrap_or_default();
    for cmd in cmds {
        let result = store.apply(cmd);
        assert!(result.is_err());
    }
}

#[test]
fn test_generator_with_done_task() {
    let mut store = create_test_store();
    store.tasks[0].status = TaskStatus::Done;
    let md = store.to_markdown();
    assert!(md.contains("- [x]"));
}

fn create_test_store() -> TaskStore {
    TaskStore {
        meta: RoadmapMeta { title: "Test Roadmap".to_string(), description: String::new() },
        sections: vec![Section {
            id: "v0.1.0".to_string(),
            title: "v0.1.0 - Foundation".to_string(),
            status: SectionStatus::Complete,
            order: 1,
        }],
        tasks: vec![
            Task {
                id: "task-one".to_string(),
                text: "First task".to_string(),
                status: TaskStatus::Pending,
                section: "v0.1.0".to_string(),
                group: Some("Test Group".to_string()),
                test: None,
                order: 1,
            },
            Task {
                id: "task-two".to_string(),
                text: "Second task".to_string(),
                status: TaskStatus::Pending,
                section: "v0.1.0".to_string(),
                group: Some("Test Group".to_string()),
                test: None,
                order: 2,
            },
        ],
    }
}