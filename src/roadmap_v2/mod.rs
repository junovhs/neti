// src/roadmap_v2/mod.rs
//! Roadmap V2 - TOML-based task management.
//!
//! V1 (ROADMAP.md) is deprecated and removed. Use tasks.toml.

pub mod cli;
pub mod generator;
pub mod parser;
pub mod storage;
pub mod types;
pub mod validation;

pub use cli::{handle_command, RoadmapV2Command};
pub use parser::parse_commands;
pub use storage::BatchResult;
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
    let result = store.apply_batch(commands)?;

    let mut messages = Vec::new();

    for warning in &result.warnings {
        messages.push(format!("? {warning}"));
    }

    if result.applied > 0 {
        store.save_with_backup(Some(roadmap_path))?;
        messages.push(format!("� Applied {} command(s)", result.applied));
    }

    for err in &result.errors {
        messages.push(format!("? {err}"));
    }

    Ok(messages)
}