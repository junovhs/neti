// src/apply/process_runner.rs
//! External command execution with streaming output.

use crate::clipboard;
use crate::spinner::SpinnerClient;
use crate::types::CommandResult;
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

pub struct CommandRunner {
    silent: bool,
}

impl CommandRunner {
    #[must_use]
    pub fn new(silent: bool) -> Self {
        Self { silent }
    }

    /// Runs a command and returns the result.
    ///
    /// # Errors
    /// Returns error if command execution fails.
    pub fn run(
        &self,
        command: &str,
        work_dir: &Path,
        client: Option<&SpinnerClient>,
    ) -> Result<CommandResult> {
        run_streaming(command, work_dir, self.silent, client)
    }
}

fn run_streaming(
    command: &str,
    work_dir: &Path,
    silent: bool,
    client: Option<&SpinnerClient>,
) -> Result<CommandResult> {
    let start = Instant::now();
    let parts: Vec<&str> = command.split_whitespace().collect();
    let Some((prog, args)) = parts.split_first() else {
        return Ok(CommandResult {
            command: command.to_string(),
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: 0,
        });
    };

    let label = extract_label(command);
    
    if let Some(c) = client {
        c.set_micro_status(format!("Running {label}..."));
    }

    let mut child = Command::new(prog)
        .args(args)
        .current_dir(work_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn {command}"))?;

    let stdout = child.stdout.take().ok_or_else(|| anyhow!("No stdout"))?;
    let stderr = child.stderr.take().ok_or_else(|| anyhow!("No stderr"))?;

    let out_acc = Arc::new(Mutex::new(String::new()));
    let err_acc = Arc::new(Mutex::new(String::new()));
    let line_count = Arc::new(AtomicUsize::new(0));

    let out_thread = spawn_reader(
        stdout,
        out_acc.clone(),
        client.cloned(),
        line_count.clone(),
    );
    let err_thread = spawn_reader(
        stderr,
        err_acc.clone(),
        client.cloned(),
        line_count.clone(),
    );

    let heartbeat = spawn_heartbeat(client.cloned(), line_count.clone());
    let status = child.wait()?;
    stop_heartbeat(heartbeat);

    let _ = out_thread.join();
    let _ = err_thread.join();

    #[allow(clippy::unwrap_used)]
    let result = CommandResult {
        command: command.to_string(),
        exit_code: status.code().unwrap_or(1),
        stdout: out_acc.lock().unwrap().clone(),
        stderr: err_acc.lock().unwrap().clone(),
        #[allow(clippy::cast_possible_truncation)]
        duration_ms: start.elapsed().as_millis() as u64,
    };

    if !status.success() && !silent {
        report_failure(&result);
    }
    Ok(result)
}

fn extract_label(command: &str) -> String {
    match command.split_whitespace().collect::<Vec<_>>().as_slice() {
        ["cargo", sub, ..] => format!("cargo {sub}"),
        ["npm", "run", s, ..] => format!("npm {s}"),
        [p, s, ..] => format!("{p} {s}"),
        [p] => (*p).to_string(),
        [] => "command".to_string(),
    }
}

fn spawn_reader<R: std::io::Read + Send + 'static>(
    input: R,
    acc: Arc<Mutex<String>>,
    client: Option<SpinnerClient>,
    count: Arc<AtomicUsize>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for line in BufReader::new(input).lines().map_while(Result::ok) {
            count.fetch_add(1, Ordering::Relaxed);
            if let Some(c) = &client {
                c.push_log(&line);
            }
            #[allow(clippy::unwrap_used)]
            {
                let mut g = acc.lock().unwrap();
                g.push_str(&line);
                g.push('\n');
            }
        }
    })
}

fn spawn_heartbeat(
    client: Option<SpinnerClient>,
    count: Arc<AtomicUsize>,
) -> Option<(Arc<AtomicBool>, thread::JoinHandle<()>)> {
    let c = client?;
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let handle = thread::spawn(move || {
        let mut last = 0;
        while r.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(200));
            let curr = count.load(Ordering::Relaxed);
            if curr == last {
                c.tick();
            }
            last = curr;
        }
    });
    Some((running, handle))
}

fn stop_heartbeat(hb: Option<(Arc<AtomicBool>, thread::JoinHandle<()>)>) {
    if let Some((running, handle)) = hb {
        running.store(false, Ordering::Relaxed);
        let _ = handle.join();
    }
}

fn report_failure(result: &CommandResult) {
    let combined = format!("{}\n{}", result.stdout, result.stderr);
    let summary = summarize(&combined);
    let label = extract_label(&result.command);

    println!();
    println!("{}", "─".repeat(60).dimmed());
    println!("{} {}", "FAILED:".red().bold(), label);
    println!("{}", "─".repeat(60).dimmed());
    println!("{summary}");
    println!("{}", "─".repeat(60).dimmed());

    if clipboard::copy_to_clipboard(&summary).is_ok() {
        println!("{}", "Error output copied to clipboard".dimmed());
    }
}

fn summarize(output: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    if lines.len() <= 30 {
        return output.to_string();
    }
    let start = lines.len().saturating_sub(30);
    format!(
        "... ({start} lines hidden)\n{}",
        lines.get(start..).unwrap_or(&[]).join("\n")
    )
}