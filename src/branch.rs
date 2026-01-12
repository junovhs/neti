// src/branch.rs
//! Git branch workflow for AI agents.

use anyhow::{Context, Result};
use std::process::Command;

const WORK_BRANCH: &str = "slopchop-work";

/// Checks if we're in a git repository.
fn in_git_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Gets the current branch name.
fn current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .context("Failed to run git")?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Checks if a branch exists.
fn branch_exists(name: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Checks if there are uncommitted changes.
fn has_uncommitted_changes() -> bool {
    Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false)
}

/// Counts modified files on the work branch.
#[must_use]
pub fn count_modified_files() -> usize {
    let output = Command::new("git").args(["status", "--porcelain"]).output();

    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).lines().count(),
        Err(_) => 0,
    }
}

/// Creates or resets the work branch.
///
/// # Errors
/// Returns error if git commands fail.
pub fn init_branch(force: bool) -> Result<BranchResult> {
    if !in_git_repo() {
        anyhow::bail!("Not a git repository. Run 'git init' first.");
    }

    let on_work_branch = current_branch()? == WORK_BRANCH;

    if branch_exists(WORK_BRANCH) && !on_work_branch {
        if force {
            // Delete and recreate
            run_git(&["branch", "-D", WORK_BRANCH])?;
        } else {
            anyhow::bail!(
                "Branch '{WORK_BRANCH}' already exists. Use --force to reset it.",
            );
        }
    }

    if on_work_branch {
        if force {
            // Reset current branch to main
            run_git(&["checkout", "main"])?;
            run_git(&["branch", "-D", WORK_BRANCH])?;
            run_git(&["checkout", "-b", WORK_BRANCH])?;
            return Ok(BranchResult::Reset);
        }
        return Ok(BranchResult::AlreadyOnBranch);
    }

    // Create and switch to work branch
    run_git(&["checkout", "-b", WORK_BRANCH])?;
    Ok(BranchResult::Created)
}

/// Promotes work branch to main.
///
/// # Errors
/// Returns error if git commands fail or checks don't pass.
pub fn promote(dry_run: bool, custom_msg: Option<String>) -> Result<PromoteResult> {
    if !in_git_repo() {
        anyhow::bail!("Not a git repository.");
    }

    let current = current_branch()?;
    if current != WORK_BRANCH {
        anyhow::bail!(
            "Not on work branch. Currently on '{current}'. Run 'slopchop branch' first.",
        );
    }

    if has_uncommitted_changes() {
        anyhow::bail!("Uncommitted changes. Commit or stash before promoting.");
    }

    if dry_run {
        return Ok(PromoteResult::DryRun);
    }

    let msg = custom_msg.unwrap_or_else(|| "chore: promote slopchop-work".to_string());

    // Merge into main
    run_git(&["checkout", "main"])?;
    run_git(&[
        "merge",
        WORK_BRANCH,
        "--no-ff",
        "-m",
        &msg,
    ])?;
    run_git(&["branch", "-d", WORK_BRANCH])?;

    Ok(PromoteResult::Merged)
}

/// Aborts work branch and returns to main.
///
/// # Errors
/// Returns error if git commands fail.
pub fn abort() -> Result<()> {
    if !in_git_repo() {
        anyhow::bail!("Not a git repository.");
    }

    let current = current_branch()?;

    if current == WORK_BRANCH {
        run_git(&["checkout", "main"])?;
    }

    if branch_exists(WORK_BRANCH) {
        run_git(&["branch", "-D", WORK_BRANCH])?;
    }

    Ok(())
}

/// Returns the name of the work branch.
#[must_use]
pub fn work_branch_name() -> &'static str {
    WORK_BRANCH
}

/// Checks if we're currently on the work branch.
#[must_use]
pub fn on_work_branch() -> bool {
    current_branch().map(|b| b == WORK_BRANCH).unwrap_or(false)
}

fn run_git(args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("Failed to run: git {}", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git {} failed: {stderr}", args.join(" "));
    }

    Ok(())
}

#[derive(Debug)]
pub enum BranchResult {
    Created,
    Reset,
    AlreadyOnBranch,
}

#[derive(Debug)]
pub enum PromoteResult {
    Merged,
    DryRun,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_branch_name() {
        assert_eq!(work_branch_name(), "slopchop-work");
    }
}