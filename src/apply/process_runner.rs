// src/apply/process_runner.rs
use crate::clipboard;
use crate::spinner::Spinner;
use crate::types::CommandResult;
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

pub struct CommandRunner {
    silent: bool,
}

impl CommandRunner {
    #[must_use]
    pub fn new(silent: bool) -> Self {
        Self { silent }
    }

    pub fn run(&self, cmd_str: &str, cwd: &Path) -> Result<CommandResult> {
        run_stage_streaming(cmd_str, cwd, self.silent)
    }
}

fn run_stage_streaming(
    cmd_str: &str,
    cwd: &Path,
    silent: bool,
) -> Result<CommandResult> {
    let start = Instant::now();
    
    // Split command
    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    let Some((prog, args)) = parts.split_first() else {
        return Ok(CommandResult {
            command: cmd_str.to_string(),
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: 0,
        });
    };

    let spinner = if silent { None } else { Some(Spinner::start(cmd_str)) };

    let mut child = Command::new(prog)
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn {cmd_str}"))?;

    let stdout = child.stdout.take().ok_or_else(|| anyhow!("Failed to open stdout"))?;
    let stderr = child.stderr.take().ok_or_else(|| anyhow!("Failed to open stderr"))?;

    // We need to capture output while updating the spinner.
    // Use threads to drain the pipes to avoid deadlocks.
    let stdout_acc = Arc::new(Mutex::new(String::new()));
    let stderr_acc = Arc::new(Mutex::new(String::new()));
    
    let out_clone = stdout_acc.clone();
    let sp_clone = spinner.clone();
    let out_thread = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            if let Some(sp) = &sp_clone {
                // Update spinner with truncated line
                let trunc = if line.len() > 60 { &line[..60] } else { &line };
                sp.set_message(format!("Running... {trunc}")); 
            }
            // Allow unwrap in helper thread; poisoning is fatal
            #[allow(clippy::unwrap_used)]
            let mut acc = out_clone.lock().unwrap();
            acc.push_str(&line);
            acc.push('\n');
        }
    });

    let err_clone = stderr_acc.clone();
    let sp_clone_err = spinner.clone();
    let err_thread = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            if let Some(sp) = &sp_clone_err {
                let trunc = if line.len() > 60 { &line[..60] } else { &line };
                sp.set_message(format!("Running... {trunc}"));
            }
            // Allow unwrap in helper thread; poisoning is fatal
            #[allow(clippy::unwrap_used)]
            let mut acc = err_clone.lock().unwrap();
            acc.push_str(&line);
            acc.push('\n');
        }
    });

    let status = child.wait()?;
    let _ = out_thread.join();
    let _ = err_thread.join();

    let success = status.success();
    if let Some(s) = spinner { s.stop(success); }

    // Use lock().unwrap().clone() instead of Arc::try_unwrap
    #[allow(clippy::unwrap_used)]
    let stdout_str = stdout_acc.lock().unwrap().clone();
    #[allow(clippy::unwrap_used)]
    let stderr_str = stderr_acc.lock().unwrap().clone();

    #[allow(clippy::cast_possible_truncation)]
    let result = CommandResult {
        command: cmd_str.to_string(),
        exit_code: status.code().unwrap_or(1),
        stdout: stdout_str,
        stderr: stderr_str,
        duration_ms: start.elapsed().as_millis() as u64,
    };

    if !success && !silent {
        let combined = format!("{}\n{}", result.stdout, result.stderr);
        let summary = summarize_output(&combined, cmd_str);
        handle_failure(cmd_str, &summary);
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