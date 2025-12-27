// src/cli/handlers.rs
use crate::analysis::RuleEngine;
use crate::apply;
use crate::apply::types::{ApplyContext, ApplyOutcome};
use crate::cli::args::ApplyArgs;
use crate::config::Config;
use crate::discovery;
use crate::exit::SlopChopExit;
use crate::pack::{self, OutputFormat, PackOptions};
use crate::prompt::PromptGenerator;
use crate::reporting;
use crate::signatures::{self, SignatureOptions};
use crate::stage;
use crate::trace::{self, TraceOptions};
use anyhow::{anyhow, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::Command;

fn get_repo_root() -> PathBuf {
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
pub fn handle_scan(verbose: bool) -> Result<SlopChopExit> {
    let mut config = Config::load();
    config.verbose = verbose;
    
    let files = discovery::discover(&config)?;
    let engine = RuleEngine::new(config);
    let report = engine.scan(files);
    
    reporting::print_report(&report)?;
    
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
pub fn handle_check() -> Result<SlopChopExit> {
    let config = Config::load();
    let repo_root = get_repo_root();
    let ctx = ApplyContext::new(&config, repo_root.clone());

    // Use effective_cwd: stage if exists, else repo root
    let cwd = stage::effective_cwd(&repo_root);

    if apply::verification::run_verification_pipeline(&ctx, &cwd)? {
        println!("{}", "[OK] All checks passed.".green().bold());
        Ok(SlopChopExit::Success)
    } else {
        Ok(SlopChopExit::CheckFailed)
    }
}

/// Handles the fix command.
///
/// # Errors
/// Returns error if command execution fails.
pub fn handle_fix() -> Result<SlopChopExit> {
    let config = Config::load();
    let Some(fix_cmds) = config.commands.get("fix") else {
        println!("No 'fix' command configured in slopchop.toml");
        return Ok(SlopChopExit::Success);
    };

    let mut failed = false;
    for cmd in fix_cmds {
        println!("Running: {cmd}");
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let Some((prog, args)) = parts.split_first() else {
            continue;
        };
        let status = Command::new(prog).args(args).status()?;
        if !status.success() {
            eprintln!("Command failed: {cmd}");
            failed = true;
        }
    }
    
    if failed {
        Ok(SlopChopExit::Error)
    } else {
        Ok(SlopChopExit::Success)
    }
}

/// Handles the dashboard command.
///
/// # Errors
/// Returns error if TUI fails to initialize or run.
pub fn handle_dashboard() -> Result<SlopChopExit> {
    let mut config = Config::load();
    crate::tui::dashboard::run(&mut config)?;
    Ok(SlopChopExit::Success)
}

/// Handles the prompt generation command.
///
/// # Errors
/// Returns error if prompt generation fails or clipboard access fails.
pub fn handle_prompt(copy: bool) -> Result<SlopChopExit> {
    let config = Config::load();
    let gen = PromptGenerator::new(config.rules);
    let prompt = gen.generate().map_err(|e| anyhow!(e.to_string()))?;

    if copy {
        crate::clipboard::copy_to_clipboard(&prompt).map_err(|e| anyhow!(e.to_string()))?;
        println!("System prompt copied to clipboard.");
    } else {
        println!("{prompt}");
    }
    Ok(SlopChopExit::Success)
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

/// Handles the trace command.
///
/// # Errors
/// Returns error if tracing fails.
pub fn handle_trace(file: &Path, depth: usize, budget: usize) -> Result<SlopChopExit> {
    let opts = TraceOptions {
        anchor: file.to_path_buf(),
        depth,
        budget,
    };
    let output = trace::run(&opts)?;
    println!("{output}");
    Ok(SlopChopExit::Success)
}

/// Handles the map command.
///
/// # Errors
/// Returns error if mapping fails.
pub fn handle_map(deps: bool) -> Result<SlopChopExit> {
    let output = trace::map(deps)?;
    println!("{output}");
    Ok(SlopChopExit::Success)
}

/// Handles the signatures command.
///
/// # Errors
/// Returns error if signature extraction fails.
pub fn handle_signatures(opts: SignatureOptions) -> Result<SlopChopExit> {
    signatures::run(&opts).map_err(|e| anyhow!(e.to_string()))?;
    Ok(SlopChopExit::Success)
}

/// Handles the apply command with CLI arguments.
///
/// # Errors
/// Returns error if application fails or IO errors occur.
pub fn handle_apply(args: &ApplyArgs) -> Result<SlopChopExit> {
    let config = Config::load();
    let repo_root = get_repo_root();
    let input = determine_input(args);

    // Handle --promote flag
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
        auto_promote: false,
        reset_stage: args.reset,
        sanitize,
    };

    let outcome = apply::run_apply(&ctx)?;
    apply::print_result(&outcome);

    Ok(map_outcome_to_exit(&outcome))
}

fn determine_sanitize(args: &ApplyArgs) -> bool {
    if args.strict { return false; }
    if args.sanitize { return true; }
    
    // Defaults:
    // File/Stdin -> Strict (raw pipe)
    // Clipboard -> Sanitize (assume browser/UI formatting)
    !matches!((args.stdin, &args.file), (true, _) | (_, Some(_)))
}

fn map_outcome_to_exit(outcome: &ApplyOutcome) -> SlopChopExit {
    match outcome {
        ApplyOutcome::Success { .. } | ApplyOutcome::Promoted { .. } | ApplyOutcome::StageReset => {
            SlopChopExit::Success
        }
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
        },
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