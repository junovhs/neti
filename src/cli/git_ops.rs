// src/cli/git_ops.rs
//! Handlers for Git-based workflow operations (branch, promote, abort).

use crate::branch;
use crate::exit::NetiExit;
use super::handlers::get_repo_root;
use anyhow::Result;
use colored::Colorize;
use std::fs;

/// Handles the branch command.
///
/// # Errors
/// Returns error if branch operations fail.
pub fn handle_branch(force: bool) -> Result<NetiExit> {
    match branch::init_branch(force)? {
        branch::BranchResult::Created => {
            println!("{}", "  Created work branch 'neti-work'".blue());
        }
        branch::BranchResult::Reset => {
            println!("{}", "  Reset work branch 'neti-work'".blue());
        }
        branch::BranchResult::AlreadyOnBranch => {
            println!("{}", "  Already on 'neti-work'".green());
        }
    }
    Ok(NetiExit::Success)
}

/// Handles the promote command.
///
/// # Errors
/// Returns error if promotion fails.
pub fn handle_promote(dry_run: bool) -> Result<NetiExit> {
    let root = get_repo_root();
    let goal_path = root.join(".neti").join("pending_goal");
    let msg = fs::read_to_string(&goal_path)
        .ok()
        .map(|s| format!("feat: {} (promoted)", s.trim()));

    match branch::promote(dry_run, msg)? {
        branch::PromoteResult::DryRun => {
            println!("{}", "[DRY RUN] Would merge 'neti-work' into main.".yellow());
        }
        branch::PromoteResult::Merged => {
            println!("{}", "  Merged 'neti-work' into main.".green().bold());
            // Clean up pending goal
            let _ = fs::remove_file(goal_path);
        }
    }
    Ok(NetiExit::Success)
}

/// Handles the abort command.
///
/// # Errors
/// Returns error if abort fails.
pub fn handle_abort() -> Result<NetiExit> {
    branch::abort()?;
    println!("{}", "  Aborted. Work branch deleted.".yellow());
    Ok(NetiExit::Success)
}
