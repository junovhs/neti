// src/apply/git.rs
use anyhow::{anyhow, Result};
use std::process::Command;
use colored::Colorize;

/// Stages all files, commits with the plan, and pushes.
///
/// # Errors
/// Returns error if git commands fail.
pub fn commit_and_push(plan: Option<&str>) -> Result<()> {
    // 1. Git Add All
    // "I always use git add . at repo root"
    run_git(&["add", "."])?;

    // 2. Check if there are changes to commit
    let status = Command::new("git").arg("status").arg("--porcelain").output()?;
    if status.stdout.is_empty() {
        println!("{}", "No changes to commit.".yellow());
        return Ok(());
    }

    // 3. Construct Commit Message
    let message = construct_message(plan);

    // 4. Git Commit
    run_git(&["commit", "-m", &message])?;
    println!("{} {}", "Git Commit:".green(), message.lines().next().unwrap_or(""));

    // 5. Git Push
    print!("{}", "Pushing to remote... ".dimmed());
    run_git(&["push"])?;
    println!("{}", "Done.".green());

    Ok(())
}

fn run_git(args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Git error: {err}"));
    }
    Ok(())
}

fn construct_message(plan: Option<&str>) -> String {
    if let Some(p) = plan {
        // We clean up the plan to make it a decent commit message
        // Remove the "GOAL:" prefix if present, as it's redundant
        let clean = p.replace("GOAL:", "").trim().to_string();
        return clean;
    }
    "warden: automated update".to_string()
}