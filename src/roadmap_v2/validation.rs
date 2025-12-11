// src/roadmap_v2/validation.rs
//! Pre-validation for roadmap commands before execution.

use super::types::{AddCommand, AfterTarget, RoadmapCommand, TaskStore};
use anyhow::Result;
use std::collections::{HashMap, HashSet};

/// Validation errors collected during pre-flight checks.
#[derive(Debug, Default)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationReport {
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Validates a batch of commands before execution.
///
/// Checks:
/// - No duplicate task IDs in ADDs
/// - No ADD with ID that already exists
/// - All AFTER targets resolve
/// - No circular AFTER chains
#[must_use]
pub fn validate_batch(store: &TaskStore, commands: &[RoadmapCommand]) -> ValidationReport {
    let mut report = ValidationReport::default();

    let add_ids = collect_add_ids(commands);
    check_duplicate_adds(&add_ids, &mut report);
    check_existing_ids(store, &add_ids, &mut report);
    check_after_targets(store, commands, &add_ids, &mut report);
    check_circular_chains(commands, &mut report);

    report
}

fn collect_add_ids(commands: &[RoadmapCommand]) -> Vec<String> {
    commands
        .iter()
        .filter_map(|cmd| {
            if let RoadmapCommand::Add(add) = cmd {
                Some(add.task.id.clone())
            } else {
                None
            }
        })
        .collect()
}

fn check_duplicate_adds(add_ids: &[String], report: &mut ValidationReport) {
    let mut seen = HashSet::new();
    for id in add_ids {
        if !seen.insert(id) {
            report
                .errors
                .push(format!("Duplicate ADD id in batch: {id}"));
        }
    }
}

fn check_existing_ids(store: &TaskStore, add_ids: &[String], report: &mut ValidationReport) {
    for id in add_ids {
        if store.tasks.iter().any(|t| &t.id == id) {
            report.errors.push(format!("Task already exists: {id}"));
        }
    }
}

fn check_after_targets(
    store: &TaskStore,
    commands: &[RoadmapCommand],
    add_ids: &[String],
    report: &mut ValidationReport,
) {
    for cmd in commands {
        let RoadmapCommand::Add(add) = cmd else {
            continue;
        };

        match &add.after {
            AfterTarget::Id(target_id) => {
                let in_store = store.tasks.iter().any(|t| &t.id == target_id);
                let in_batch = add_ids.contains(target_id);
                if !in_store && !in_batch {
                    let suggestion = suggest_similar(store, target_id);
                    let msg = format!("AFTER target not found: {target_id}{suggestion}");
                    report.errors.push(msg);
                }
            }
            AfterTarget::Text(needle) => {
                let found = store.tasks.iter().any(|t| t.text.contains(needle));
                if !found {
                    report
                        .warnings
                        .push(format!("AFTER TEXT may not match: \"{needle}\""));
                }
            }
            AfterTarget::End | AfterTarget::Previous | AfterTarget::Line(_) => {}
        }
    }
}

fn suggest_similar(store: &TaskStore, target: &str) -> String {
    let target_lower = target.to_lowercase();

    for task in &store.tasks {
        if task.id.to_lowercase().contains(&target_lower) {
            return format!(" (did you mean '{}'?)", task.id);
        }
    }

    String::new()
}

fn check_circular_chains(commands: &[RoadmapCommand], report: &mut ValidationReport) {
    let mut deps: HashMap<String, String> = HashMap::new();

    for cmd in commands {
        let RoadmapCommand::Add(add) = cmd else {
            continue;
        };

        if let AfterTarget::Id(ref target) = add.after {
            deps.insert(add.task.id.clone(), target.clone());
        }
    }

    for start in deps.keys() {
        if has_cycle(&deps, start) {
            report
                .errors
                .push(format!("Circular AFTER chain detected involving: {start}"));
            break;
        }
    }
}

fn has_cycle(deps: &HashMap<String, String>, start: &str) -> bool {
    let mut visited = HashSet::new();
    let mut current = start;

    while let Some(next) = deps.get(current) {
        if !visited.insert(next.as_str()) {
            return true;
        }
        current = next;
    }

    false
}

/// Resolves AFTER targets to concrete order positions.
///
/// # Errors
/// Returns error if a target cannot be resolved.
pub fn resolve_position(
    store: &TaskStore,
    add: &AddCommand,
    last_added_order: Option<usize>,
) -> Result<usize> {
    match &add.after {
        AfterTarget::End => Ok(next_order_in_section(store, &add.task.section)),
        AfterTarget::Previous => {
            let base = last_added_order.unwrap_or(0);
            Ok(base + 1)
        }
        AfterTarget::Id(target_id) => {
            let task = store
                .tasks
                .iter()
                .find(|t| &t.id == target_id)
                .ok_or_else(|| anyhow::anyhow!("AFTER target not found: {target_id}"))?;
            Ok(task.order + 1)
        }
        AfterTarget::Text(needle) => {
            let task = store
                .tasks
                .iter()
                .find(|t| t.text.contains(needle))
                .ok_or_else(|| anyhow::anyhow!("No task contains text: \"{needle}\""))?;
            Ok(task.order + 1)
        }
        AfterTarget::Line(n) => Ok(*n),
    }
}

fn next_order_in_section(store: &TaskStore, section: &str) -> usize {
    store
        .tasks
        .iter()
        .filter(|t| t.section == section)
        .map(|t| t.order)
        .max()
        .map_or(0, |m| m + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::roadmap_v2::types::{Task, TaskStatus};

    fn make_store() -> TaskStore {
        TaskStore {
            tasks: vec![Task {
                id: "existing-task".to_string(),
                text: "Existing task".to_string(),
                status: TaskStatus::Pending,
                section: "v1".to_string(),
                test: None,
                group: None,
                order: 0,
            }],
            ..Default::default()
        }
    }

    #[test]
    fn test_duplicate_add_detected() {
        let store = make_store();
        let commands = vec![
            RoadmapCommand::Add(AddCommand {
                task: Task {
                    id: "new-1".to_string(),
                    text: "A".to_string(),
                    status: TaskStatus::Pending,
                    section: "v1".to_string(),
                    test: None,
                    group: None,
                    order: 0,
                },
                after: AfterTarget::End,
            }),
            RoadmapCommand::Add(AddCommand {
                task: Task {
                    id: "new-1".to_string(),
                    text: "B".to_string(),
                    status: TaskStatus::Pending,
                    section: "v1".to_string(),
                    test: None,
                    group: None,
                    order: 0,
                },
                after: AfterTarget::End,
            }),
        ];

        let report = validate_batch(&store, &commands);
        assert!(!report.is_ok());
        assert!(report.errors[0].contains("Duplicate ADD"));
    }

    #[test]
    fn test_existing_id_rejected() {
        let store = make_store();
        let commands = vec![RoadmapCommand::Add(AddCommand {
            task: Task {
                id: "existing-task".to_string(),
                text: "X".to_string(),
                status: TaskStatus::Pending,
                section: "v1".to_string(),
                test: None,
                group: None,
                order: 0,
            },
            after: AfterTarget::End,
        })];

        let report = validate_batch(&store, &commands);
        assert!(!report.is_ok());
    }
}