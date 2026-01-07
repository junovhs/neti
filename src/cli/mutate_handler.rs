// src/cli/mutate_handler.rs
use crate::exit::SlopChopExit;
use crate::mutate::{self, MutateOptions};
use crate::cli::handlers::get_repo_root;
use anyhow::Result;

/// Handles the mutate command.
///
/// # Errors
/// Returns error if mutation testing fails.
pub fn handle_mutate(
    workers: Option<usize>,
    timeout: u64,
    json: bool,
    filter: Option<String>,
) -> Result<SlopChopExit> {
    let opts = MutateOptions {
        workers,
        timeout_secs: timeout,
        json,
        filter,
    };
    
    let repo_root = get_repo_root();
    let report = mutate::run(&repo_root, &opts)?;
    
    if report.summary.survived > 0 {
        Ok(SlopChopExit::CheckFailed)
    } else {
        Ok(SlopChopExit::Success)
    }
}
