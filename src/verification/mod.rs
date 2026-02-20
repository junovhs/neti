//! External command verification pipeline.
//!
//! Runs commands defined in `[commands]` section of neti.toml
//! and captures output to `neti-report.txt`.

mod runner;

use std::path::Path;

use crate::config::Config;
pub use runner::run_commands;

/// Result of a single command execution.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CommandResult {
    /// The command that was executed.
    pub command: String,
    /// Whether the command succeeded (exit code 0).
    pub passed: bool,
    /// Exit code, if available.
    pub exit_code: Option<i32>,
    /// Combined stdout and stderr output.
    pub output: String,
    /// Execution time in milliseconds.
    pub duration_ms: u64,
}

impl CommandResult {
    #[must_use]
    pub fn new(
        command: String,
        passed: bool,
        exit_code: Option<i32>,
        output: String,
        duration_ms: u64,
    ) -> Self {
        Self {
            command,
            passed,
            exit_code,
            output,
            duration_ms,
        }
    }

    /// Count error lines in output.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.output
            .lines()
            .filter(|line| {
                let lower = line.to_lowercase();
                lower.contains("error:") || lower.contains("error[") || lower.starts_with("error")
            })
            .count()
    }

    /// Count warning lines in output.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.output
            .lines()
            .filter(|line| {
                let lower = line.to_lowercase();
                lower.contains("warning:")
                    || lower.contains("warn:")
                    || lower.starts_with("warning")
            })
            .count()
    }
}

/// Result of running the verification pipeline.
#[derive(Debug, serde::Serialize)]
pub struct VerificationReport {
    /// Whether all commands passed.
    pub passed: bool,
    /// Combined output from all commands.
    pub output: String,
    /// Individual command results.
    pub commands: Vec<CommandResult>,
    /// Total execution time in milliseconds.
    pub duration_ms: u64,
}

impl VerificationReport {
    #[must_use]
    pub fn new(
        passed: bool,
        output: String,
        commands: Vec<CommandResult>,
        duration_ms: u64,
    ) -> Self {
        Self {
            passed,
            output,
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
        self.commands.iter().filter(|c| c.passed).count()
    }

    /// Number of commands that failed.
    #[must_use]
    pub fn failed_count(&self) -> usize {
        self.commands.iter().filter(|c| !c.passed).count()
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
