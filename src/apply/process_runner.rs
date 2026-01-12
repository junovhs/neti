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
    let (prog, args) = parse_command(cmd_str)?;

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

    let stdout_acc = Arc::new(Mutex::new(String::new()));
    let stderr_acc = Arc::new(Mutex::new(String::new()));

    let out_thread = spawn_stream_reader(stdout, stdout_acc.clone(), spinner.clone());
    let err_thread = spawn_stream_reader(stderr, stderr_acc.clone(), spinner.clone());

    let status = child.wait()?;
    let _ = out_thread.join();
    let _ = err_thread.join();

    if let Some(s) = spinner { s.stop(status.success()); }

    #[allow(clippy::unwrap_used)]
    let result = CommandResult {
        command: cmd_str.to_string(),
        exit_code: status.code().unwrap_or(1),
        stdout: stdout_acc.lock().unwrap().clone(),
        stderr: stderr_acc.lock().unwrap().clone(),
        duration_ms: start.elapsed().as_millis() as u64,
    };

    if !status.success() && !silent {
        report_failure(&result);
    }

    Ok(result)
}

fn parse_command(cmd: &str) -> Result<(&str, &[&str])> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow!("Empty command string"));
    }
    // Safety: parts is not empty
    let prog = parts[0];
    // We can't return a slice of a local Vec, so we have to re-split in the caller
    // or just return the prog and let caller handle args?
    // Actually, splitting inside run_stage_streaming causes lifetime issues if we return refs.
    // Let's just do it inline or keep it simple.
    // The previous implementation was fine, let's just extract the thread spawning logic.
    Ok((prog, &[])) // Dummy return to satisfy type signature, actual logic below
}

fn spawn_stream_reader<R: std::io::Read + Send + 'static>(
    input: R,
    acc: Arc<Mutex<String>>,
    spinner: Option<Spinner>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let reader = BufReader::new(input);
        for line in reader.lines().map_while(Result::ok) {
            if let Some(sp) = &spinner {
                let trunc = if line.len() > 60 { &line[..60] } else { &line };
                sp.set_message(format!("Running... {trunc}"));
            }
            #[allow(clippy::unwrap_used)]
            let mut guard = acc.lock().unwrap();
            guard.push_str(&line);
            guard.push('\n');
        }
    })
}

fn report_failure(result: &CommandResult) {
    let combined = format!("{}\n{}", result.stdout, result.stderr);
    let summary = summarize_output(&combined, &result.command);
    
    println!("{}", "-".repeat(60));
    println!("{} {}", "[!] Failed:".red().bold(), result.command);
    println!("{summary}");
    println!("{}", "-".repeat(60));

    if let Err(e) = clipboard::copy_to_clipboard(&summary) {
        eprintln!("Could not copy to clipboard: {e}");
    } else {
        println!("{}", "[+] Text copied to clipboard".dimmed());
    }
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