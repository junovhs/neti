// src/roadmap_v2/mod.rs
//! Roadmap V2 - TOML-based task management.
//!
//! V1 (ROADMAP.md) is deprecated and removed. Use tasks.toml.

pub mod cli;
pub mod generator;
pub mod parser;
pub mod storage;
pub mod types;

pub use parser::parse_commands;
pub use types::{RoadmapCommand, Task, TaskStatus, TaskStore};

use anyhow::Result;
use std::path::Path;

/// Handles roadmap commands from AI output.
///
/// # Errors
/// Returns error if parsing or applying commands fails.
pub fn handle_input(roadmap_path: &Path, content: &str) -> Result<Vec<String>> {
    let commands = parser::parse_commands(content)?;

    if commands.is_empty() {
        return Ok(vec![]);
    }

    let mut store = TaskStore::load(Some(roadmap_path))?;
    let mut results = Vec::new();

    for cmd in commands {
        let description = describe_command(&cmd);
        match store.apply(cmd) {
            Ok(()) => results.push(format!("✓ {description}")),
            Err(e) => results.push(format!("✗ {description}: {e}")),
        }
    }

    store.save(Some(roadmap_path))?;

    Ok(results)
}

fn describe_command(cmd: &RoadmapCommand) -> String {
    match cmd {
        RoadmapCommand::Check { id } => format!("Checked: {id}"),
        RoadmapCommand::Uncheck { id } => format!("Unchecked: {id}"),
        RoadmapCommand::Add(task) => format!("Added: {} ({})", task.id, task.section),
        RoadmapCommand::Update { id, .. } => format!("Updated: {id}"),
        RoadmapCommand::Delete { id } => format!("Deleted: {id}"),
    }
}