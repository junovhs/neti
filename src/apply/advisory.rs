// src/apply/advisory.rs
use crate::branch;
use colored::Colorize;
use std::path::Path;

/// Threshold for triggering the high edit volume advisory.
const NAG_THRESHOLD: usize = 3;

/// Prints an advisory if many files have been modified on the work branch.
pub fn maybe_print_edit_advisory(repo_root: &Path) {
    // Only check if we are actually in a git repo and on the work branch (or about to be)
    // The branch module handles the logic of counting modifications.
    // We pass repo_root just in case, though branch uses cwd/git commands usually.
    let _ = repo_root; // Silence unused warning if we don't use it directly

    let modified_count = branch::count_modified_files();

    if modified_count > NAG_THRESHOLD {
        println!();
        println!("{}", "━".repeat(60).yellow());
        println!("{}", "[ADVISORY] High Edit Volume Detected".yellow().bold());
        println!("  {modified_count} files modified.");
        println!("  Consider committing soon to maintain high-integrity checkpoints.");
        println!("  Run: {} to commit changes.", "git commit".cyan());
        println!("{}", "━".repeat(60).yellow());
    }
}