// src/apply/git.rs
use anyhow::{anyhow, Result};
use colored::Colorize;
use std::process::Command;

/// Stages all files, commits with the provided message, and pushes.
///
/// # Errors
/// Returns error if git commands fail.
pub fn commit_and_push(message: &str) -> Result<()> {
    // 1. Git Add All
    run_git(&["add", "."])?;

    // 2. Check if there are changes to commit
    let status = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()?;
    if status.stdout.is_empty() {
        println!("{}", "No changes to commit.".yellow());
        return Ok(());
    }

    // 3. Git Commit
    let final_message = clean_message(message);
    run_git(&["commit", "-m", &final_message])?;
    println!(
        "{} {}",
        "Git Commit:".green(),
        final_message.lines().next().unwrap_or("")
    );

    // 4. Git Push
    print!("{}", "Pushing to remote... ".dimmed());
    run_git(&["push"])?;
    println!("{}", "Done.".green());

    Ok(())
}

fn run_git(args: &[&str]) -> Result<()> {
    let output = Command::new("git").args(args).output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Git error: {err}"));
    }
    Ok(())
}

fn clean_message(raw: &str) -> String {
    let clean = raw.replace("GOAL:", "").trim().to_string();
    if clean.is_empty() {
        "slopchop: automated update".to_string()
    } else {
        clean
    }
}
