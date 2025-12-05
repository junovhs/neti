// src/roadmap/mod.rs
pub mod audit;
pub mod cli;
pub mod cmd_handlers;
pub mod cmd_helpers;
pub mod cmd_parser;
pub mod cmd_runner;
pub mod diff;
pub mod display;
pub mod parser;
pub mod prompt;
pub mod str_utils;
pub mod types;

// Re-exports for external use
pub use cmd_runner::apply_commands;
pub use parser::slugify;
pub use prompt::{generate_prompt, PromptOptions};
pub use types::{ApplyResult, Command, CommandBatch, Roadmap, TaskStatus};

/// Handles roadmap command input from clipboard/file.
///
/// # Errors
/// Returns error if roadmap file cannot be read or written.
pub fn handle_input(
    roadmap_path: &std::path::Path,
    content: &str,
) -> anyhow::Result<Vec<String>> {
    let roadmap_content = std::fs::read_to_string(roadmap_path)?;
    let mut roadmap = Roadmap::parse(&roadmap_content);
    roadmap.path = Some(roadmap_path.to_string_lossy().to_string());

    let batch = CommandBatch::parse(content);
    let results = apply_commands(&mut roadmap, &batch);

    std::fs::write(roadmap_path, &roadmap.raw)?;
    Ok(results.into_iter().map(|r| r.to_string()).collect())
}