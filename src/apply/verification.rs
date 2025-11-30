// src/apply/verification.rs
use crate::apply::types::ApplyContext;
use anyhow::Result;
use colored::Colorize;
use std::fmt::Write as FmtWrite;
use std::process::Command;

/// Runs configured checks and Warden scan to verify application.
/// Returns `(success, log_output)`.
///
/// # Errors
/// Returns error if command execution fails.
pub fn verify_application(ctx: &ApplyContext) -> Result<(bool, String)> {
    println!("{}", "\n🔍 Verifying changes...".blue().bold());
    let mut log_buffer = String::new();

    if let Some(commands) = ctx.config.commands.get("check") {
        for cmd in commands {
            let (success, output) = run_check_command(cmd)?;
            let _ = writeln!(log_buffer, "> {cmd}\n{output}");

            if !success {
                return Ok((false, log_buffer));
            }
        }
    }

    println!("Running structural scan...");
    let (success, output) = run_warden_check()?;
    let _ = writeln!(log_buffer, "> warden scan\n{output}");

    Ok((success, log_buffer))
}

fn run_check_command(cmd: &str) -> Result<(bool, String)> {
    println!("Running check: {}", cmd.dimmed());
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let Some((prog, args)) = parts.split_first() else {
        return Ok((true, String::new()));
    };

    let output = Command::new(prog).args(args).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    print!("{stdout}");
    eprint!("{stderr}");

    let combined = format!("{stdout}\n{stderr}");
    Ok((output.status.success(), combined))
}

fn run_warden_check() -> Result<(bool, String)> {
    let output = Command::new("warden").output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    print!("{stdout}");
    eprint!("{stderr}");

    let combined = format!("{stdout}\n{stderr}");
    Ok((output.status.success(), combined))
}