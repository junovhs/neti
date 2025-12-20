// src/cli/handlers.rs
use crate::apply;
use crate::apply::types::ApplyContext;
use crate::cli::args::ApplyArgs;
use crate::config::Config;
use crate::pack::{self, OutputFormat, PackOptions};
use crate::prompt::PromptGenerator;
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

/// Handles the check command.
///
/// # Errors
/// Returns error if verification pipeline fails.
pub fn handle_check() -> Result<()> {
    let config = Config::load();
    let repo_root = get_repo_root();
    let ctx = ApplyContext::new(&config, repo_root.clone());

    // Use effective_cwd: stage if exists, else repo root
    let cwd = stage::effective_cwd(&repo_root);

    if apply::verification::run_verification_pipeline(&ctx, &cwd)? {
        println!("{}", "[OK] All checks passed.".green().bold());
        Ok(())
    } else {
        std::process::exit(1);
    }
}

/// Handles the fix command.
///
/// # Errors
/// Returns error if command execution fails.
pub fn handle_fix() -> Result<()> {
    let config = Config::load();
    let Some(fix_cmds) = config.commands.get("fix") else {
        println!("No 'fix' command configured in slopchop.toml");
        return Ok(());
    };

    for cmd in fix_cmds {
        println!("Running: {cmd}");
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let Some((prog, args)) = parts.split_first() else {
            continue;
        };
        let status = Command::new(prog).args(args).status()?;
        if !status.success() {
            eprintln!("Command failed: {cmd}");
        }
    }
    Ok(())
}

/// Handles the dashboard command.
///
/// # Errors
/// Returns error if TUI fails.
pub fn handle_dashboard() -> Result<()> {
    let mut config = Config::load();
    crate::tui::dashboard::run(&mut config)?;
    Ok(())
}

/// Handles the prompt generation command.
///
/// # Errors
/// Returns error if prompt generation fails or clipboard access fails.
pub fn handle_prompt(copy: bool) -> Result<()> {
    let config = Config::load();
    let gen = PromptGenerator::new(config.rules);
    let prompt = gen.generate().map_err(|e| anyhow!(e.to_string()))?;

    if copy {
        crate::clipboard::copy_to_clipboard(&prompt).map_err(|e| anyhow!(e.to_string()))?;
        println!("System prompt copied to clipboard.");
    } else {
        println!("{prompt}");
    }
    Ok(())
}

/// Handles the pack command.
///
/// # Errors
/// Returns error if packing fails.
pub fn handle_pack(args: PackArgs) -> Result<()> {
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
    Ok(())
}

/// Handles the trace command.
///
/// # Errors
/// Returns error if tracing fails.
pub fn handle_trace(file: &Path, depth: usize, budget: usize) -> Result<()> {
    let opts = TraceOptions {
        anchor: file.to_path_buf(),
        depth,
        budget,
    };
    let output = trace::run(&opts)?;
    println!("{output}");
    Ok(())
}

/// Handles the map command.
///
/// # Errors
/// Returns error if mapping fails.
pub fn handle_map(deps: bool) -> Result<()> {
    let output = trace::map(deps)?;
    println!("{output}");
    Ok(())
}

/// Handles the signatures command.
///
/// # Errors
/// Returns error if extraction fails.
pub fn handle_signatures(opts: SignatureOptions) -> Result<()> {
    signatures::run(&opts).map_err(|e| anyhow!(e.to_string()))?;
    Ok(())
}

/// Handles the apply command with CLI arguments.
///
/// # Errors
/// Returns error if application fails.
pub fn handle_apply(args: &ApplyArgs) -> Result<()> {
    let config = Config::load();
    let repo_root = get_repo_root();
    let input = determine_input(args);

    // Handle --promote flag
    if args.promote {
        let ctx = ApplyContext::new(&config, repo_root);
        let outcome = apply::run_promote(&ctx)?;
        apply::print_result(&outcome);
        return Ok(());
    }

    let ctx = ApplyContext {
        config: &config,
        repo_root: repo_root.clone(),
        force: args.force,
        dry_run: args.dry_run,
        input,
        check_after: args.check,
        auto_promote: false,
        reset_stage: args.reset,
    };

    let outcome = apply::run_apply(&ctx)?;
    apply::print_result(&outcome);

    Ok(())
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
