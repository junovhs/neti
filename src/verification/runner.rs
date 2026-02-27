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
///
/// Uses POSIX shell-style quoting rules via `shell_words::split` so that
/// commands like `cargo clippy -- -D "some flag"` are parsed correctly.
fn run_single_command(repo_root: &Path, cmd_str: &str) -> CommandResult {
    let start = Instant::now();

    let parts = match shell_words::split(cmd_str) {
        Ok(p) => p,
        Err(e) => {
            return CommandResult::new(
                cmd_str.to_string(),
                -1,
                String::new(),
                format!("Failed to parse command: {e}"),
                0,
            );
        }
    };

    let Some(program) = parts.first() else {
        return CommandResult::new(
            cmd_str.to_string(),
            -1,
            String::new(),
            "Empty command".to_string(),
            0,
        );
    };
    let args = &parts[1..];

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    // --- run_single_command: shell parsing ---

    #[test]
    fn simple_command_executes() {
        let result = run_single_command(&repo_root(), "echo hello");
        assert!(result.passed());
        assert!(result.stdout().contains("hello"));
    }

    #[test]
    fn double_quoted_args_preserved() {
        // Without shell-words, "hello world" would split into two args
        let result = run_single_command(&repo_root(), "echo \"hello world\"");
        assert!(result.passed());
        assert!(result.stdout().contains("hello world"));
    }

    #[test]
    fn single_quoted_args_preserved() {
        let result = run_single_command(&repo_root(), "echo 'hello world'");
        assert!(result.passed());
        assert!(result.stdout().contains("hello world"));
    }

    #[test]
    fn unclosed_quote_returns_parse_error() {
        let result = run_single_command(&repo_root(), "echo \"unterminated");
        assert!(!result.passed());
        assert_eq!(result.exit_code(), -1);
        assert!(result.stderr().contains("Failed to parse command"));
    }

    #[test]
    fn empty_command_returns_error() {
        let result = run_single_command(&repo_root(), "");
        assert!(!result.passed());
        assert_eq!(result.exit_code(), -1);
        assert!(result.stderr().contains("Empty command"));
    }

    #[test]
    fn whitespace_only_command_returns_error() {
        let result = run_single_command(&repo_root(), "   ");
        assert!(!result.passed());
        assert_eq!(result.exit_code(), -1);
        assert!(result.stderr().contains("Empty command"));
    }

    #[test]
    fn nonexistent_program_returns_error() {
        let result = run_single_command(&repo_root(), "nonexistent_binary_xyz_123");
        assert!(!result.passed());
        assert_eq!(result.exit_code(), -1);
        assert!(result.stderr().contains("Failed to execute"));
    }

    #[test]
    fn failing_command_captures_exit_code() {
        let result = run_single_command(&repo_root(), "false");
        assert!(!result.passed());
        assert_ne!(result.exit_code(), 0);
    }

    #[test]
    fn stderr_captured_separately() {
        // `ls` on a nonexistent path writes to stderr and exits non-zero
        let result =
            run_single_command(&repo_root(), "ls /nonexistent_path_that_does_not_exist_xyz");
        assert!(!result.passed());
        assert!(!result.stderr().is_empty());
    }

    #[test]
    fn multiple_args_with_mixed_quoting() {
        // echo receives three args: "a b", "c", "d e"
        let result = run_single_command(&repo_root(), "echo \"a b\" c 'd e'");
        assert!(result.passed());
        let out = result.stdout();
        assert!(out.contains("a b"));
        assert!(out.contains("c"));
        assert!(out.contains("d e"));
    }

    // --- run_commands: pipeline behavior ---

    #[test]
    fn run_commands_empty_list_passes() {
        let report = run_commands(&repo_root(), &[], |_, _, _| {});
        assert!(report.passed);
        assert_eq!(report.total_commands(), 0);
    }

    #[test]
    fn run_commands_all_pass() {
        let cmds = vec!["echo one".to_string(), "echo two".to_string()];
        let report = run_commands(&repo_root(), &cmds, |_, _, _| {});
        assert!(report.passed);
        assert_eq!(report.total_commands(), 2);
        assert_eq!(report.passed_count(), 2);
    }

    #[test]
    fn run_commands_one_failure_fails_report() {
        let cmds = vec!["echo ok".to_string(), "false".to_string()];
        let report = run_commands(&repo_root(), &cmds, |_, _, _| {});
        assert!(!report.passed);
        assert_eq!(report.total_commands(), 2);
        assert_eq!(report.failed_count(), 1);
    }

    #[test]
    fn run_commands_callback_receives_correct_indices() {
        let cmds = vec!["echo a".to_string(), "echo b".to_string()];
        let mut calls = Vec::new();
        let _ = run_commands(&repo_root(), &cmds, |cmd, idx, total| {
            calls.push((cmd.to_string(), idx, total));
        });
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0], ("echo a".to_string(), 1, 2));
        assert_eq!(calls[1], ("echo b".to_string(), 2, 2));
    }

    #[test]
    fn run_commands_with_quoted_args() {
        let cmds = vec!["echo \"hello world\"".to_string()];
        let report = run_commands(&repo_root(), &cmds, |_, _, _| {});
        assert!(report.passed);
        assert!(report.commands[0].stdout().contains("hello world"));
    }
}
