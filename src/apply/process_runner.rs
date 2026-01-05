// src/apply/process_runner.rs
use crate::clipboard;
use crate::spinner::Spinner;
use crate::types::CommandResult;
use anyhow::Result;
use colored::Colorize;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

pub struct CommandRunner {
    silent: bool,
}

impl CommandRunner {
    #[must_use]
    pub fn new(silent: bool) -> Self {
        Self { silent }
    }

    /// Runs a command in the specified directory.
    ///
    /// # Errors
    /// Returns error if command execution fails.
    pub fn run(&self, cmd_str: &str, cwd: &Path) -> Result<CommandResult> {
        run_stage_in_dir(cmd_str, cmd_str, cwd, self.silent)
    }
}

fn run_stage_in_dir(
    label: &str,
    cmd_str: &str,
    cwd: &Path,
    silent: bool,
) -> Result<CommandResult> {
    let sp = if silent { None } else { Some(Spinner::start(label)) };
    let start = Instant::now();

    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    let Some((prog, args)) = parts.split_first() else {
        if let Some(s) = sp { s.stop(true); }
        return Ok(CommandResult {
            command: cmd_str.to_string(),
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: 0,
        });
    };

    let output = Command::new(prog).args(args).current_dir(cwd).output()?;
    let duration = start.elapsed();
    let success = output.status.success();

    if let Some(s) = sp { s.stop(success); }

    #[allow(clippy::cast_possible_truncation)]
    let result = CommandResult {
        command: cmd_str.to_string(),
        exit_code: output.status.code().unwrap_or(1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        duration_ms: duration.as_millis() as u64,
    };

    if !success && !silent {
        let combined = format!("{}\n{}", result.stdout, result.stderr);
        let summary = summarize_output(&combined, cmd_str);
        handle_failure(label, &summary);
    }

    Ok(result)
}

fn summarize_output(output: &str, cmd: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let max_lines = 20;

    if lines.len() <= max_lines {
        return output.to_string();
    }

    let summary: String = lines.iter().take(max_lines).copied().collect::<Vec<_>>().join("\n");
    format!(
        "{summary}\n... ({} more lines, run '{cmd}' for full output)",
        lines.len() - max_lines
    )
}

fn handle_failure(label: &str, summary: &str) {
    println!("{}", "-".repeat(60));
    println!("{} {label}", "[!] Failed:".red().bold());
    println!("{summary}");
    println!("{}", "-".repeat(60));

    if let Err(e) = clipboard::copy_to_clipboard(summary) {
        eprintln!("Could not copy to clipboard: {e}");
    } else {
        println!("{}", "[+] Text copied to clipboard".dimmed());
    }
}
