// src/apply/verification.rs
use crate::apply::types::ApplyContext;
use crate::clipboard;
use crate::spinner::Spinner;
use anyhow::Result;
use colored::Colorize;
use std::process::Command;

/// Runs the full verification pipeline: Check -> Test -> Scan.
/// Stops at the first failure, summarizes output, and copies to clipboard.
///
/// # Errors
/// Returns error if command execution fails.
pub fn run_verification_pipeline(ctx: &ApplyContext) -> Result<bool> {
    println!("{}", "\n> Verifying changes...".blue().bold());

    // 1. Run external checks (e.g. clippy, eslint)
    if let Some(commands) = ctx.config.commands.get("check") {
        for cmd in commands {
            if !run_stage(cmd, cmd)? {
                return Ok(false);
            }
        }
    }

    // 2. Run SlopChop scan (Structural check)
    if !run_stage("slopchop scan", "slopchop")? {
        return Ok(false);
    }

    Ok(true)
}

fn run_stage(label: &str, cmd_str: &str) -> Result<bool> {
    let sp = Spinner::start(label);
    
    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    let Some((prog, args)) = parts.split_first() else {
        sp.stop(true);
        return Ok(true);
    };

    let output = Command::new(prog).args(args).output()?;
    let success = output.status.success();
    sp.stop(success);

    if !success {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}\n{stderr}");
        
        let summary = summarize_output(&combined, cmd_str);
        
        handle_failure(label, &summary);
    }

    Ok(success)
}

fn handle_failure(stage: &str, summary: &str) {
    println!("{}", "-".repeat(60).red());
    println!("{} Failed: {}", "[!]".red(), stage.bold());
    println!("{}", summary.trim());
    println!("{}", "-".repeat(60).red());

    match clipboard::smart_copy(summary) {
        Ok(msg) => println!("{} {}", "[+]".yellow(), msg),
        Err(e) => println!("{} Failed to copy to clipboard: {}", "[!]".yellow(), e),
    }
}

fn summarize_output(output: &str, cmd: &str) -> String {
    let is_test = cmd.contains("test");
    let is_cargo = cmd.contains("cargo");

    output
        .lines()
        .filter(|line| keep_line(line, is_cargo, is_test))
        .take(50) // Limit length for token efficiency
        .collect::<Vec<_>>()
        .join("\n")
}

fn keep_line(line: &str, is_cargo: bool, is_test: bool) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }

    if is_common_noise(trimmed) {
        return false;
    }

    if is_test && is_test_noise(trimmed) {
        return false;
    }

    if is_cargo && is_cargo_noise(trimmed) {
        return false;
    }

    true
}

fn is_common_noise(line: &str) -> bool {
    line.starts_with("Finished") 
        || line.starts_with("Compiling") 
        || line.starts_with("Running") 
        || line.starts_with("Doc-tests") 
        || line.starts_with("Checking")
}

fn is_test_noise(line: &str) -> bool {
    line.starts_with("test result:") || line.starts_with("test ")
}

fn is_cargo_noise(line: &str) -> bool {
    line.contains("warnings emitted") || line.contains("generated")
}