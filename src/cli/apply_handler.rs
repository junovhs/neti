// src/cli/apply_handler.rs
//! Handler for the apply command.

use crate::apply;
use crate::apply::types::{ApplyContext, ApplyInput, ApplyOutcome};
use crate::cli::args::ApplyArgs;
use crate::config::Config;
use crate::exit::SlopChopExit;
use super::handlers::get_repo_root;
use anyhow::Result;
use colored::Colorize;

/// Handles the apply command with CLI arguments.
///
/// # Errors
/// Returns error if application fails or IO errors occur.
pub fn handle_apply(args: &ApplyArgs) -> Result<SlopChopExit> {
    let config = Config::load();
    let repo_root = get_repo_root();
    let input = determine_input(args);

    if args.sync {
        println!("{}", "Sync is deprecated. Use git branches instead.".yellow());
        return Ok(SlopChopExit::Success);
    }

    if args.promote {
        let ctx = ApplyContext::new(&config, repo_root);
        let outcome = apply::run_promote(&ctx)?;
        apply::print_result(&outcome);
        return Ok(map_outcome_to_exit(&outcome));
    }

    let sanitize = determine_sanitize(args);

    let ctx = ApplyContext {
        config: &config,
        repo_root: repo_root.clone(),
        force: args.force,
        dry_run: args.dry_run,
        input,
        check_after: args.check,
        auto_promote: config.preferences.auto_promote,
        reset_stage: args.reset,
        sanitize,
        silent: false,
    };

    let outcome = apply::run_apply(&ctx)?;
    apply::print_result(&outcome);

    Ok(map_outcome_to_exit(&outcome))
}

fn determine_sanitize(args: &ApplyArgs) -> bool {
    if args.strict {
        return false;
    }
    if args.sanitize {
        return true;
    }
    !matches!((args.stdin, &args.file), (true, _) | (_, Some(_)))
}

fn map_outcome_to_exit(outcome: &ApplyOutcome) -> SlopChopExit {
    match outcome {
        ApplyOutcome::Success { .. }
        | ApplyOutcome::Promoted { .. }
        | ApplyOutcome::StageReset => SlopChopExit::Success,
        ApplyOutcome::ValidationFailure { .. } => SlopChopExit::InvalidInput,
        ApplyOutcome::ParseError(msg) => {
            if msg.contains("Patch mismatch")
                || msg.contains("Ambiguous")
                || msg.contains("Could not find exact match")
            {
                SlopChopExit::PatchFailure
            } else {
                SlopChopExit::InvalidInput
            }
        }
        ApplyOutcome::WriteError(_) => SlopChopExit::Error,
    }
}

fn determine_input(args: &ApplyArgs) -> ApplyInput {
    if args.stdin {
        ApplyInput::Stdin
    } else if let Some(ref path) = args.file {
        ApplyInput::File(path.clone())
    } else {
        ApplyInput::Clipboard
    }
}