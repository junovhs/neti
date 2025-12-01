pub mod audit;
pub mod cli;
pub mod cmd_handlers;
pub mod cmd_parser;
pub mod cmd_runner;
pub mod diff;
pub mod display;
pub mod parser;
pub mod prompt;
pub mod str_utils;
pub mod types;

// Re-export CommandBatch from types
pub use cmd_runner::apply_commands;
pub use parser::slugify;
pub use prompt::{generate_prompt, PromptOptions};
pub use types::CommandBatch;
pub use types::*;

use std::path::Path;
use anyhow::{Context, Result};

/// Parses input for roadmap commands and applies them to the specified file.
/// Returns a list of result messages (Success/Error).
///
/// # Errors
/// Returns error if file IO fails.
pub fn handle_input(file_path: &Path, input: &str) -> Result<Vec<String>> {
    // 1. Check if input actually contains a roadmap block
    let batch = CommandBatch::parse(input);
    if batch.commands.is_empty() {
        return Ok(Vec::new());
    }

    // 2. Load the roadmap (or error if missing)
    let mut roadmap = Roadmap::from_file(file_path)
        .context(format!("Failed to load roadmap from {}", file_path.display()))?;

    // 3. Apply commands
    let results = apply_commands(&mut roadmap, &batch);

    // 4. Save if any changes succeeded
    let any_success = results.iter().any(|r| matches!(r, ApplyResult::Success(_)));
    if any_success {
        roadmap.save(file_path)?;
    }

    // 5. Convert results to strings
    Ok(results.into_iter().map(|r| r.to_string()).collect())
}