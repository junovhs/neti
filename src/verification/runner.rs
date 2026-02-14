//! Command execution and output capture.

use std::fmt::Write;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use super::{CommandResult, VerificationReport};

/// Runs a list of commands and captures combined output.
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
    let mut combined_output = String::new();
    let mut results = Vec::new();
    let total = commands.len();

    for (idx, cmd_str) in commands.iter().enumerate() {
        on_command(cmd_str, idx + 1, total);
        let _ = writeln!(combined_output, "$ {cmd_str}");

        let cmd_start = Instant::now();
        let result = run_single_command(repo_root, cmd_str);

        #[allow(clippy::cast_possible_truncation)]
        let cmd_duration = cmd_start.elapsed().as_millis() as u64;

        match result {
            Ok(cmd_output) => {
                combined_output.push_str(&cmd_output);
                combined_output.push('\n');

                let passed = !cmd_output.contains("[exit code:")
                    || cmd_output.lines().any(|l| l.contains("[exit code: 0]"));

                let exit_code = cmd_output.lines().find_map(|l| {
                    l.strip_prefix("[exit code: ")
                        .and_then(|s| s.strip_suffix(']'))
                        .and_then(|s| s.parse::<i32>().ok())
                });

                let passed = exit_code.map_or(passed, |c| c == 0);

                results.push(CommandResult::new(
                    cmd_str.clone(),
                    passed,
                    exit_code,
                    cmd_output.clone(),
                    cmd_duration,
                ));

                if !passed {
                    all_passed = false;
                }
            }
            Err(e) => {
                all_passed = false;
                let _ = writeln!(combined_output, "ERROR: {e}");
                combined_output.push('\n');

                results.push(CommandResult::new(
                    cmd_str.clone(),
                    false,
                    None,
                    format!("ERROR: {e}"),
                    cmd_duration,
                ));
            }
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    let total_duration = start.elapsed().as_millis() as u64;

    VerificationReport::new(all_passed, combined_output, results, total_duration)
}

/// Runs a single command string.
fn run_single_command(repo_root: &Path, cmd_str: &str) -> anyhow::Result<String> {
    let parts: Vec<&str> = cmd_str.split_whitespace().collect();

    let Some(&program) = parts.first() else {
        return Ok("(empty command)".to_string());
    };

    let args = parts.get(1..).unwrap_or(&[]);

    let output = Command::new(program)
        .args(args)
        .current_dir(repo_root)
        .output()?;

    let mut result = String::new();

    if !output.stdout.is_empty() {
        result.push_str(&String::from_utf8_lossy(&output.stdout));
    }

    if !output.stderr.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&String::from_utf8_lossy(&output.stderr));
    }

    if !output.status.success() {
        let _ = writeln!(
            result,
            "[exit code: {}]",
            output.status.code().unwrap_or(-1)
        );
    }

    Ok(result)
}
