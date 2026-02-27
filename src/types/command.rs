//! Result of an external command execution.

use serde::Serialize;

/// Result of an external command execution.
#[derive(Debug, Clone, Serialize)]
pub struct CommandResult {
    // neti:allow(LCOM4) Data record, one getter per field.
    /// The command that was executed (display form).
    command: String,
    /// Whether the command succeeded (exit code 0).
    passed: bool,
    /// Process exit code (-1 if unavailable, e.g., killed by signal).
    exit_code: i32,
    /// Standard output.
    stdout: String,
    /// Standard error.
    stderr: String,
    /// Execution time in milliseconds.
    duration_ms: u64,
}

impl CommandResult {
    /// Creates a new command result.
    #[must_use]
    pub fn new(
        command: String,
        exit_code: i32,
        stdout: String,
        stderr: String,
        duration_ms: u64,
    ) -> Self {
        Self {
            command,
            passed: exit_code == 0,
            exit_code,
            stdout,
            stderr,
            duration_ms,
        }
    }

    /// The command that was executed.
    #[must_use]
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Whether the command succeeded (exit code 0).
    #[must_use]
    pub fn passed(&self) -> bool {
        self.passed
    }

    /// Process exit code.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }

    /// Standard output.
    #[must_use]
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    /// Standard error.
    #[must_use]
    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    /// Combined stdout and stderr output.
    #[must_use]
    pub fn output(&self) -> String {
        if self.stderr.is_empty() {
            self.stdout.clone()
        } else if self.stdout.is_empty() {
            self.stderr.clone()
        } else {
            format!("{}\n{}", self.stdout, self.stderr)
        }
    }

    /// Execution time in milliseconds.
    #[must_use]
    pub fn duration_ms(&self) -> u64 {
        self.duration_ms
    }

    /// Count error lines in output.
    #[must_use]
    pub fn error_count(&self) -> usize {
        count_matching_lines(&self.output(), |lower| {
            lower.contains("error:") || lower.contains("error[") || lower.starts_with("error")
        })
    }

    /// Count warning lines in output.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        count_matching_lines(&self.output(), |lower| {
            lower.contains("warning:") || lower.contains("warn:") || lower.starts_with("warning")
        })
    }
}

/// Counts lines in `text` where `predicate` matches the lowercased line.
fn count_matching_lines(text: &str, predicate: impl Fn(&str) -> bool) -> usize {
    text.lines()
        .filter(|line| predicate(&line.to_lowercase()))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passed_true_when_exit_code_zero() {
        let r = CommandResult::new("echo hello".into(), 0, "hello\n".into(), String::new(), 5);
        assert!(r.passed());
        assert_eq!(r.exit_code(), 0);
    }

    #[test]
    fn passed_false_when_exit_code_one() {
        let r = CommandResult::new("false".into(), 1, String::new(), String::new(), 5);
        assert!(!r.passed());
        assert_eq!(r.exit_code(), 1);
    }

    #[test]
    fn passed_false_when_exit_code_negative() {
        let r = CommandResult::new("killed".into(), -1, String::new(), String::new(), 5);
        assert!(!r.passed());
        assert_eq!(r.exit_code(), -1);
    }

    #[test]
    fn output_returns_stdout_when_stderr_empty() {
        let r = CommandResult::new("cmd".into(), 0, "stdout only".into(), String::new(), 0);
        assert_eq!(r.output(), "stdout only");
    }

    #[test]
    fn output_returns_stderr_when_stdout_empty() {
        let r = CommandResult::new("cmd".into(), 1, String::new(), "stderr only".into(), 0);
        assert_eq!(r.output(), "stderr only");
    }

    #[test]
    fn output_combines_stdout_and_stderr() {
        let r = CommandResult::new("cmd".into(), 0, "out".into(), "err".into(), 0);
        let combined = r.output();
        assert!(combined.contains("out"), "should contain stdout");
        assert!(combined.contains("err"), "should contain stderr");
        assert!(combined.starts_with("out"), "stdout should come first");
    }

    #[test]
    fn error_count_detects_error_colon() {
        let r = CommandResult::new(
            "cargo".into(),
            1,
            "error: cannot find\nerror: another\nwarning: something".into(),
            String::new(),
            0,
        );
        assert_eq!(r.error_count(), 2);
    }

    #[test]
    fn error_count_detects_error_bracket() {
        let r = CommandResult::new(
            "cargo".into(),
            1,
            "error[E0432]: unresolved import".into(),
            String::new(),
            0,
        );
        assert_eq!(r.error_count(), 1);
    }

    #[test]
    fn error_count_detects_line_starting_with_error() {
        let r = CommandResult::new(
            "cmd".into(),
            1,
            "error\nError\nERROR".into(),
            String::new(),
            0,
        );
        assert_eq!(r.error_count(), 3);
    }

    #[test]
    fn error_count_zero_when_clean() {
        let r = CommandResult::new("cmd".into(), 0, "all good".into(), String::new(), 0);
        assert_eq!(r.error_count(), 0);
    }

    #[test]
    fn warning_count_detects_warning_colon() {
        let r = CommandResult::new(
            "cargo".into(),
            0,
            "warning: unused variable\nwarning: dead code".into(),
            String::new(),
            0,
        );
        assert_eq!(r.warning_count(), 2);
    }

    #[test]
    fn warning_count_detects_warn_colon() {
        let r = CommandResult::new(
            "eslint".into(),
            0,
            "warn: something".into(),
            String::new(),
            0,
        );
        assert_eq!(r.warning_count(), 1);
    }

    #[test]
    fn warning_count_zero_when_clean() {
        let r = CommandResult::new("cmd".into(), 0, "all good".into(), String::new(), 0);
        assert_eq!(r.warning_count(), 0);
    }

    #[test]
    fn accessors_return_correct_values() {
        let r = CommandResult::new(
            "cargo test".into(),
            0,
            "test passed".into(),
            "some debug".into(),
            1234,
        );
        assert_eq!(r.command(), "cargo test");
        assert_eq!(r.stdout(), "test passed");
        assert_eq!(r.stderr(), "some debug");
        assert_eq!(r.duration_ms(), 1234);
    }

    #[test]
    fn error_and_warning_counts_scan_combined_output() {
        let r = CommandResult::new(
            "cmd".into(),
            1,
            "error: in stdout".into(),
            "warning: in stderr".into(),
            0,
        );
        assert_eq!(r.error_count(), 1, "should find error in stdout");
        assert_eq!(r.warning_count(), 1, "should find warning in stderr");
    }
}
