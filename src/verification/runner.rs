//! Command execution and output capture.

use super::VerificationReport;
use crate::types::CommandResult;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

/// Runs a list of commands and captures output.
///
/// The `on_command` callback is invoked before each command executes,
/// allowing for progress feedback.
#[must_use]
pub fn run_commands<F>(
    repo_root: &Path,
    commands: &[String],
    mut on_command: F,
) -> VerificationReport
where
    F: FnMut(&str, usize, usize),
{
    let start = Instant::now();
    let mut all_passed = true;
    let mut results = Vec::new();
    let total = commands.len();

    for (idx, cmd_str) in commands.iter().enumerate() {
        on_command(cmd_str, idx + 1, total);

        let result = run_single_command(repo_root, cmd_str);

        if !result.passed() {
            all_passed = false;
        }

        results.push(result);
    }

    let total_duration = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    VerificationReport::new(all_passed, results, total_duration)
}

/// Runs a single command string and captures stdout/stderr separately.
fn run_single_command(repo_root: &Path, cmd_str: &str) -> CommandResult {
    let start = Instant::now();

    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    let Some(&program) = parts.first() else {
        return CommandResult::new(
            cmd_str.to_string(),
            -1,
            String::new(),
            "Empty command".to_string(),
            0,
        );
    };
    let args = parts.get(1..).unwrap_or(&[]);

    let output = Command::new(program)
        .args(args)
        .current_dir(repo_root)
        .output();

    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

    match output {
        Ok(output) => {
            let exit_code = output.status.code().unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            CommandResult::new(cmd_str.to_string(), exit_code, stdout, stderr, duration_ms)
        }
        Err(e) => CommandResult::new(
            cmd_str.to_string(),
            -1,
            String::new(),
            format!("Failed to execute: {e}"),
            duration_ms,
        ),
    }
}
