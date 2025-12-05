// src/apply/verification.rs
use crate::apply::types::ApplyContext;
use anyhow::Result;
use colored::Colorize;
use std::fmt::Write as FmtWrite;
use std::io::{self, Write};
use std::process::Command;

/// Runs configured checks and `SlopChop` scan to verify application.
/// Returns `(success, log_output)`.
///
/// # Errors
/// Returns error if command execution fails.
pub fn verify_application(ctx: &ApplyContext) -> Result<(bool, String)> {
    println!("{}", "\n?? Verifying changes...".blue().bold());
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
    let (success, output) = run_slopchop_check()?;
    let _ = writeln!(log_buffer, "> slopchop scan\n{output}");

    Ok((success, log_buffer))
}

fn run_check_command(cmd: &str) -> Result<(bool, String)> {
    print!("   {} {} ", "".blue(), cmd.dimmed());
    let _ = io::stdout().flush();

    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let Some((prog, args)) = parts.split_first() else {
        println!("{}", "�".green());
        return Ok((true, String::new()));
    };

    let output = Command::new(prog).args(args).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    if output.status.success() {
        println!("{}", "�".green());
        Ok((true, combined))
    } else {
        println!("{}", "?".red());
        print!("{stdout}");
        eprint!("{stderr}");
        Ok((false, combined))
    }
}

fn run_slopchop_check() -> Result<(bool, String)> {
    let output = Command::new("slopchop").output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    // We print slopchop output because it contains summary stats (tokens, etc.)
    print!("{stdout}");
    eprint!("{stderr}");

    Ok((output.status.success(), combined))
}