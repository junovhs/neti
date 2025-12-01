// src/roadmap/diff.rs
use crate::roadmap::types::{Command, MovePosition, Roadmap, Task};
use std::collections::HashMap;

/// Compares two roadmaps and generates a "Wicked Smart" patch of commands.
#[must_use]
pub fn diff(current: &Roadmap, incoming: &Roadmap) -> Vec<Command> {
    let mut commands = Vec::new();
    
    // 1. Structural Scan: Add missing sections
    let curr_sections: HashMap<String, String> = current.sections.iter()
        .map(|s| (s.id.clone(), s.heading.clone()))
        .collect();
        
    for section in &incoming.sections {
        if !curr_sections.contains_key(&section.id) {
            commands.push(Command::AddSection {
                heading: section.heading.clone(),
            });
        }
    }

    // 2. Task Analysis
    let curr_map = map_tasks_with_parent(current);
    let inc_map = map_tasks_with_parent(incoming);

    // 2a. Updates, Moves, Checks
    for (id, (curr_task, curr_parent)) in &curr_map {
        if let Some((inc_task, inc_parent)) = inc_map.get(id) {
            let ctx = TaskComparisonContext {
                id,
                curr: curr_task,
                inc: inc_task,
                curr_parent,
                inc_parent,
            };
            compare_task_detailed(&ctx, &mut commands);
        } else {
            commands.push(Command::Delete { path: id.clone() });
        }
    }

    // 2b. Additions
    for (id, (inc_task, inc_parent_title)) in &inc_map {
        if !curr_map.contains_key(id) {
            commands.push(Command::Add {
                parent: inc_parent_title.clone(),
                text: inc_task.text.clone(),
                after: None, 
            });
        }
    }

    commands
}

struct TaskComparisonContext<'a> {
    id: &'a str,
    curr: &'a Task,
    inc: &'a Task,
    curr_parent: &'a str,
    inc_parent: &'a str,
}

fn compare_task_detailed(ctx: &TaskComparisonContext, cmds: &mut Vec<Command>) {
    detect_move(ctx, cmds);
    detect_status_change(ctx, cmds);
    detect_text_change(ctx, cmds);
}

fn detect_move(ctx: &TaskComparisonContext, cmds: &mut Vec<Command>) {
    if ctx.curr_parent != ctx.inc_parent {
        cmds.push(Command::Move {
            path: ctx.id.to_string(),
            position: MovePosition::EndOfSection(ctx.inc_parent.to_string()),
        });
    }
}

fn detect_status_change(ctx: &TaskComparisonContext, cmds: &mut Vec<Command>) {
    if ctx.curr.status != ctx.inc.status {
        match ctx.inc.status {
            crate::roadmap::TaskStatus::Complete => cmds.push(Command::Check { path: ctx.id.to_string() }),
            crate::roadmap::TaskStatus::Pending => cmds.push(Command::Uncheck { path: ctx.id.to_string() }),
        }
    }
}

fn detect_text_change(ctx: &TaskComparisonContext, cmds: &mut Vec<Command>) {
    if ctx.curr.text != ctx.inc.text {
        cmds.push(Command::Update {
            path: ctx.id.to_string(),
            text: ctx.inc.text.clone(),
        });
    }
}

type TaskMap<'a> = HashMap<String, (&'a Task, String)>;

fn map_tasks_with_parent(roadmap: &Roadmap) -> TaskMap<'_> {
    let mut map = HashMap::new();
    for section in &roadmap.sections {
        for task in &section.tasks {
            map.insert(task.id.clone(), (task, section.heading.clone()));
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::roadmap::types::{Section, TaskStatus};

    fn make_roadmap(sections: Vec<Section>) -> Roadmap {
        Roadmap {
            path: None,
            title: "Test".into(),
            sections,
            raw: String::new(),
        }
    }

    fn make_section(title: &str, tasks: Vec<Task>) -> Section {
        Section {
            id: crate::roadmap::slugify(title),
            heading: title.into(),
            level: 2,
            theme: None,
            tasks,
            subsections: vec![],
            raw_content: String::new(),
            line_start: 0,
            line_end: 0,
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
    fn test_diff_move_section() {
        let t1 = make_task("t1", TaskStatus::Pending);
        
        let sec_a = make_section("Section A", vec![t1.clone()]);
        let sec_b = make_section("Section B", vec![]);
        let curr = make_roadmap(vec![sec_a, sec_b]);

        // Incoming: t1 moved to B
        let new_sec_a = make_section("Section A", vec![]);
        let new_sec_b = make_section("Section B", vec![t1]);
        let inc = make_roadmap(vec![new_sec_a, new_sec_b]);

        let cmds = diff(&curr, &inc);
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            Command::Move { path, position } => {
                assert_eq!(path, "t1");
                assert_eq!(*position, MovePosition::EndOfSection("Section B".into()));
            },
            _ => panic!("Expected MOVE"),
        }
    }

    #[test]
    fn test_diff_add_section() {
        let curr = make_roadmap(vec![]);
        let inc = make_roadmap(vec![make_section("New Era", vec![])]);
        
        let cmds = diff(&curr, &inc);
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            Command::AddSection { heading } => assert_eq!(heading, "New Era"),
            _ => panic!("Expected AddSection"),
        }
    }
}