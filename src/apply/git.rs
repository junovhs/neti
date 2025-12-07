// src/apply/git.rs
use anyhow::{bail, Context, Result};
use std::process::Command;

/// Checks if the git working tree has uncommitted changes.
///
/// # Errors
/// Returns error if git command fails.
pub fn is_dirty() -> Result<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to run git status")?;

    if !output.status.success() {
        bail!("git status failed");
    }

    Ok(!output.stdout.is_empty())
}

/// Checks if we're inside a git repository.
#[must_use]
pub fn in_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Commits all staged changes and pushes to remote.
///
/// # Errors
/// Returns error if any git command fails.
pub fn commit_and_push(message: &str) -> Result<()> {
    run_git(&["add", "-A"])?;

    // Check if there's anything to commit
    let status = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .status()
        .context("Failed to check staged changes")?;

    if status.success() {
        // Nothing staged, nothing to commit
        return Ok(());
    }

    run_git(&["commit", "-m", message])?;

    // Push, but don't fail if no remote configured
    let push_result = Command::new("git")
        .args(["push"])
        .output()
        .context("Failed to run git push")?;

    if !push_result.status.success() {
        let stderr = String::from_utf8_lossy(&push_result.stderr);
        if stderr.contains("No configured push destination")
            || stderr.contains("no upstream branch")
            || stderr.contains("does not have any commits")
        {
            // No remote configured, that's fine
            return Ok(());
        }
        bail!("git push failed: {}", stderr.trim());
    }

    Ok(())
}

fn run_git(args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("Failed to run git {}", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }

    Ok(())
}