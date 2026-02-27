//! External command verification pipeline.
//!
//! Runs commands defined in `[commands]` section of neti.toml
//! and captures output to `neti-report.txt`.

mod runner;

use std::path::Path;

use crate::config::Config;

// Re-export the canonical CommandResult from types
pub use crate::types::CommandResult;
pub use runner::run_commands;

/// Result of running the verification pipeline.
#[derive(Debug, serde::Serialize)]
pub struct VerificationReport {
    /// Whether all commands passed.
    pub passed: bool,
    /// Individual command results.
    pub commands: Vec<CommandResult>,
    /// Total execution time in milliseconds.
    pub duration_ms: u64,
}

impl VerificationReport {
    #[must_use]
    pub fn new(passed: bool, commands: Vec<CommandResult>, duration_ms: u64) -> Self {
        Self {
            passed,
            commands,
            duration_ms,
        }
    }

    /// Total number of commands run.
    #[must_use]
    pub fn total_commands(&self) -> usize {
        self.commands.len()
    }

    /// Number of commands that passed.
    #[must_use]
    pub fn passed_count(&self) -> usize {
        self.commands.iter().filter(|c| c.passed()).count()
    }

    /// Number of commands that failed.
    #[must_use]
    pub fn failed_count(&self) -> usize {
        self.commands.iter().filter(|c| !c.passed()).count()
    }

    /// Total errors across all command outputs.
    #[must_use]
    pub fn total_errors(&self) -> usize {
        self.commands.iter().map(CommandResult::error_count).sum()
    }

    /// Total warnings across all command outputs.
    #[must_use]
    pub fn total_warnings(&self) -> usize {
        self.commands.iter().map(CommandResult::warning_count).sum()
    }
}

/// Runs the verification pipeline using commands from config.
///
/// The `on_command` callback is invoked before each command executes.
pub fn run<F>(repo_root: &Path, on_command: F) -> VerificationReport
where
    F: FnMut(&str, usize, usize),
{
    let config = Config::load();
    let commands = config.commands.get("check").cloned().unwrap_or_default();

    runner::run_commands(repo_root, &commands, on_command)
}
