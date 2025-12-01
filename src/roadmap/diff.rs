// src/roadmap/diff.rs
use crate::roadmap::types::{Command, Roadmap, Task};
use std::collections::HashMap;

/// Compares the current roadmap on disk with an incoming roadmap (rewritten by AI).
/// Generates a list of commands to transform `current` into `incoming`.
///
/// This allows Warden to reject a full file rewrite while still understanding
/// what the AI wanted to do, ensuring safe, atomic updates.
#[must_use]
pub fn diff(current: &Roadmap, incoming: &Roadmap) -> Vec<Command> {
    let mut commands = Vec::new();
    let curr_tasks = map_tasks(current);
    let inc_tasks = map_tasks(incoming);

    commands.extend(detect_changes(&curr_tasks, &inc_tasks));
    commands.extend(detect_additions(&curr_tasks, incoming));

    commands
}

fn detect_changes(curr: &HashMap<String, &Task>, inc: &HashMap<String, &Task>) -> Vec<Command> {
    let mut cmds = Vec::new();
    for (id, curr_task) in curr {
        if let Some(inc_task) = inc.get(id) {
            compare_task(id, curr_task, inc_task, &mut cmds);
        } else {
            // Task in Current but NOT in Incoming -> Deleted
            cmds.push(Command::Delete { path: id.clone() });
        }
    }
    cmds
}

fn compare_task(id: &str, curr: &Task, inc: &Task, cmds: &mut Vec<Command>) {
    // Status Change
    if curr.status != inc.status {
        match inc.status {
            crate::roadmap::TaskStatus::Complete => {
                cmds.push(Command::Check {
                    path: id.to_string(),
                });
            }
            crate::roadmap::TaskStatus::Pending => {
                cmds.push(Command::Uncheck {
                    path: id.to_string(),
                });
            }
        }
    }

    // Text Change
    if curr.text != inc.text {
        cmds.push(Command::Update {
            path: id.to_string(),
            text: inc.text.clone(),
        });
    }
}

fn detect_additions(curr: &HashMap<String, &Task>, incoming: &Roadmap) -> Vec<Command> {
    let mut cmds = Vec::new();
    // V1 Strategy: Iterate top-level sections and their tasks.
    // Deeply nested new tasks are not currently auto-detected for ADD (limitation accepted for now).
    for section in &incoming.sections {
        for task in &section.tasks {
            if !curr.contains_key(&task.id) {
                cmds.push(Command::Add {
                    parent: section.heading.clone(),
                    text: task.text.clone(),
                    after: None,
                });
            }
        }
    }
    cmds
}

fn map_tasks(roadmap: &Roadmap) -> HashMap<String, &Task> {
    let mut map = HashMap::new();
    for task in roadmap.all_tasks() {
        map.insert(task.id.clone(), task);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::roadmap::types::{Section, TaskStatus};

    fn make_dummy_roadmap(tasks: Vec<Task>) -> Roadmap {
        Roadmap {
            path: None,
            title: "Test".into(),
            sections: vec![Section {
                id: "main".into(),
                heading: "Main".into(),
                level: 2,
                theme: None,
                tasks,
                subsections: vec![],
                raw_content: String::new(),
                line_start: 0,
                line_end: 0,
            }],
            raw: String::new(),
        }
    }

    fn make_task(id: &str, status: TaskStatus) -> Task {
        Task {
            id: id.into(),
            path: id.into(),
            text: format!("Task {id}"),
            status,
            indent: 0,
            line: 0,
            children: vec![],
            tests: vec![],
        }
    }

    #[test]
    fn test_diff_status_change() {
        let t1_curr = make_task("t1", TaskStatus::Pending);
        let t1_inc = make_task("t1", TaskStatus::Complete);

        let curr = make_dummy_roadmap(vec![t1_curr]);
        let inc = make_dummy_roadmap(vec![t1_inc]);

        let cmds = diff(&curr, &inc);
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            Command::Check { path } => assert_eq!(path, "t1"),
            _ => panic!("Wrong command"),
        }
    }

    #[test]
    fn test_diff_new_task() {
        let curr = make_dummy_roadmap(vec![]);
        let t1 = make_task("new-task", TaskStatus::Pending);
        let inc = make_dummy_roadmap(vec![t1]);

        let cmds = diff(&curr, &inc);
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            Command::Add { parent, text, .. } => {
                assert_eq!(parent, "Main");
                assert_eq!(text, "Task new-task");
            }
            _ => panic!("Wrong command"),
        }
    }
}