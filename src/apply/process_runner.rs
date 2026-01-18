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

    let display_cmd = if cmd_str.len() > 50 {
        format!("{}...", &cmd_str[..47])
    } else {
        cmd_str.to_string()
    };

    // The spinner now handles the Triptych HUD (Macro/Micro/Atomic)
    let spinner = if silent { None } else { Some(Spinner::start(display_cmd)) };

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

    // IO Threads push directly to the HUD
    let out_thread = spawn_stream_reader(stdout, stdout_acc.clone(), spinner.clone());
    let err_thread = spawn_stream_reader(stderr, stderr_acc.clone(), spinner.clone());

    let status = child.wait()?;
    let _ = out_thread.join();
    let _ = err_thread.join();

    if let Some(s) = spinner { s.stop(status.success()); }

    #[allow(clippy::cast_possible_truncation)]
    let duration = start.elapsed().as_millis() as u64;

    #[allow(clippy::unwrap_used)]
    let result = CommandResult {
        command: cmd_str.to_string(),
        exit_code: status.code().unwrap_or(1),
        stdout: stdout_acc.lock().unwrap().clone(),
        stderr: stderr_acc.lock().unwrap().clone(),
        duration_ms: duration,
    };

    if !status.success() && !silent {
        report_failure(&result);
    }

    Ok(result)
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
                sp.push_log(&line);
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
    let max_lines = 30; 

    if lines.len() <= max_lines {
        return output.to_string();
    }

    let start_idx = lines.len().saturating_sub(max_lines);
    let summary = lines.get(start_idx..).unwrap_or(&[]).join("\n");
    
    format!(
        "... ({start_idx} lines hidden, run '{cmd}' for full output)\n{summary}"
    )
}