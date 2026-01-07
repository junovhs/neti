// src/cli/handlers.rs
use crate::analysis::RuleEngine;
use crate::apply;
use crate::apply::types::{ApplyContext, ApplyOutcome};
use crate::cli::args::ApplyArgs;
use crate::config::Config;
use crate::discovery;
use crate::exit::SlopChopExit;
use crate::map;
use crate::pack::{self, OutputFormat, PackOptions};
use crate::reporting;
use crate::stage;
use crate::signatures::{self, SignatureOptions};
use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;
#[must_use]
pub fn get_repo_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct PackArgs {
    pub stdout: bool,
    pub copy: bool,
    pub noprompt: bool,
    pub format: OutputFormat,
    pub skeleton: bool,
    pub code_only: bool,
    pub verbose: bool,
    pub target: Option<PathBuf>,
    pub focus: Vec<PathBuf>,
    pub depth: usize,
}

/// Handles the scan command.
///
/// # Errors
/// Returns error if discovery or scanning fails.
pub fn handle_scan(verbose: bool, locality: bool, json: bool) -> Result<SlopChopExit> {
    if locality {
        return super::locality::handle_locality();
    }

    let mut config = Config::load();
    config.verbose = verbose;

    let files = discovery::discover(&config)?;
    let engine = RuleEngine::new(config);
    let report = engine.scan(files);

    if json {
        reporting::print_json(&report)?;
    } else {
        reporting::print_report(&report)?;
    }

    if report.has_errors() {
        Ok(SlopChopExit::CheckFailed)
    } else {
        Ok(SlopChopExit::Success)
    }
}

/// Handles the check command.
///
/// # Errors
/// Returns error if verification pipeline fails to execute.
pub fn handle_check(json: bool) -> Result<SlopChopExit> {
    let config = Config::load();
    let repo_root = get_repo_root();
    let mut ctx = ApplyContext::new(&config, repo_root.clone());
    ctx.silent = json;

    let cwd = stage::effective_cwd(&repo_root);

    let report = apply::verification::run_verification_pipeline(&ctx, &cwd)?;

    if json {
        reporting::print_json(&report)?;
    } else if report.passed {
        println!("{}", "[OK] All checks passed.".green().bold());
    }

    if report.passed {
        Ok(SlopChopExit::Success)
    } else {
        Ok(SlopChopExit::CheckFailed)
    }
}

/// Handles the pack command.
///
/// # Errors
/// Returns error if packing fails.
pub fn handle_pack(args: PackArgs) -> Result<SlopChopExit> {
    let opts = PackOptions {
        stdout: args.stdout,
        copy: args.copy,
        verbose: args.verbose,
        prompt: !args.noprompt,
        format: args.format,
        skeleton: args.skeleton,
        code_only: args.code_only,
        target: args.target,
        focus: args.focus,
        depth: args.depth,
    };
    pack::run(&opts)?;
    Ok(SlopChopExit::Success)
}

/// Handles the map command.
///
/// # Errors
/// Returns error if mapping fails.
pub fn handle_map(deps: bool) -> Result<SlopChopExit> {
    let output = map::generate(deps)?;
    println!("{output}");
    Ok(SlopChopExit::Success)
}

/// Handles the signatures command.
///
/// # Errors
/// Returns error if signature extraction fails.
pub fn handle_signatures(opts: SignatureOptions) -> Result<SlopChopExit> {
    signatures::run(&opts)?;
    Ok(SlopChopExit::Success)
}

/// Handles the config command.
///
/// # Errors
/// Returns error if config editing fails.
pub fn handle_config() -> Result<SlopChopExit> {
    crate::cli::config_ui::run_config_editor()?;
    Ok(SlopChopExit::Success)
}

/// Handles the sabotage command.
///
/// # Errors
/// Returns error if sabotage fails.
pub fn handle_sabotage(file: &std::path::Path) -> Result<SlopChopExit> {
    let repo_root = get_repo_root();
    let mut stage = stage::StageManager::new(&repo_root);
    
    if !stage.exists() {
        // Auto-create stage if missing for convenience?
        // The brief says "Must ONLY operate on .slopchop/stage/worktree".
        // If stage doesn't exist, we can't operate on it.
        println!("{}", "No stage found. Run 'slopchop apply' or create a stage first.".yellow());
        return Ok(SlopChopExit::Error);
    }
    
    // We might need to ensure the file exists in the stage.
    // The user might pass a path relative to repo root, we need to map it.
    // sabotage_file expects path relative to worktree root? 
    // Actually sabotage_file takes "path" and joins it with worktree.
    // So if user passes "src/main.rs", it looks for ".slopchop/stage/worktree/src/main.rs".
    
    let report = crate::analysis::sabotage::sabotage_file(file, &mut stage)?;
    
    if report.mutated {
        println!("{}", "Sabotage successful!".green().bold());
        println!("  File: {}", report.file.display());
        println!("  Line: {}", report.line);
        println!("  Mutation: '{}' -> '{}'", report.original_op.red(), report.new_op.green());
        println!("\nRun 'slopchop check' to see if tests catch this bug!");
        Ok(SlopChopExit::Success)
    } else {
        println!("{}", "No mutable logic found in file.".yellow());
        Ok(SlopChopExit::Error)
    }
}

/// Handles the apply command with CLI arguments.
///
/// # Errors
/// Returns error if application fails or IO errors occur.
pub fn handle_apply(args: &ApplyArgs) -> Result<SlopChopExit> {
    let config = Config::load();
    let repo_root = get_repo_root();
    let input = determine_input(args);

    if args.sync {
        return super::stage_handlers::handle_sync(&repo_root);
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

fn determine_input(args: &ApplyArgs) -> apply::types::ApplyInput {
    if args.stdin {
        apply::types::ApplyInput::Stdin
    } else if let Some(ref path) = args.file {
        apply::types::ApplyInput::File(path.clone())
    } else {
        apply::types::ApplyInput::Clipboard
    }
}

// Re-export from stage_handlers for backwards compatibility
pub use super::stage_handlers::handle_stage;

