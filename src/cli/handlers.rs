// src/cli/handlers.rs
use crate::analysis::RuleEngine;
use crate::apply;
use crate::apply::types::ApplyContext;
use crate::config::Config;
use crate::error::Result;
use crate::pack::{self, OutputFormat, PackOptions};
use crate::prompt::PromptGenerator;
use crate::reporting;
use crate::trace::{self, TraceOptions};
use std::path::{Path, PathBuf};
use std::process::Command;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct PackArgs {
    pub stdout: bool,
    pub copy: bool,
    pub noprompt: bool,
    pub format: OutputFormat,
    pub skeleton: bool,
    pub git_only: bool,
    pub no_git: bool,
    pub code_only: bool,
    pub verbose: bool,
    pub target: Option<PathBuf>,
    pub focus: Vec<PathBuf>,
    pub depth: usize,
}

/// Handles the initialization command.
///
/// # Errors
/// Returns error if directory change fails or wizard fails.
pub fn handle_init(path: Option<PathBuf>) -> Result<()> {
    if let Some(target) = path {
        std::env::set_current_dir(target)?;
    }
    crate::wizard::run()?;
    Ok(())
}

/// Handles the check command.
///
/// # Errors
/// Returns error if discovery, analysis, or external commands fail.
pub fn handle_check() -> Result<()> {
    let mut config = Config::new();
    config.load_local_config();

    // 1. Run external check commands (cargo test, clippy, etc.)
    println!("> Running 'check' pipeline...");
    if let Some(check_cmds) = config.commands.get("check") {
        for cmd in check_cmds {
            run_check_command(cmd)?;
        }
    }

    // 2. Run internal structural scan
    println!("> Running structural scan...");
    let engine = RuleEngine::new(config.clone());
    let files = crate::discovery::discover(&config)?;
    let report = engine.scan(files);

    reporting::print_report(&report)?;

    if report.has_errors() {
        std::process::exit(1);
    }
    Ok(())
}

fn run_check_command(cmd: &str) -> Result<()> {
    print!("   > {cmd} ");
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    
    let Some((prog, args)) = parts.split_first() else {
        return Ok(());
    };

    let status = Command::new(prog).args(args).status()?;
    if status.success() {
        println!("ok");
        Ok(())
    } else {
        println!("err");
        Err(crate::error::SlopChopError::Other(format!(
            "Command failed: {cmd}"
        )))
    }
}

/// Handles the fix command.
///
/// # Errors
/// Returns error if command execution fails.
pub fn handle_fix() -> Result<()> {
    let mut config = Config::new();
    config.load_local_config();

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
    let mut config = Config::new();
    config.load_local_config();
    crate::tui::dashboard::run(&mut config)?;
    Ok(())
}

/// Handles the prompt generation command.
///
/// # Errors
/// Returns error if prompt generation fails or clipboard access fails.
pub fn handle_prompt(copy: bool) -> Result<()> {
    let mut config = Config::new();
    config.load_local_config();
    let gen = PromptGenerator::new(config.rules);
    let prompt = gen.generate().map_err(|e| crate::error::SlopChopError::Other(e.to_string()))?;
    
    if copy {
        crate::clipboard::copy_to_clipboard(&prompt).map_err(|e| crate::error::SlopChopError::Other(e.to_string()))?;
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
        git_only: args.git_only,
        no_git: args.no_git,
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

/// Handles the apply command.
///
/// # Errors
/// Returns error if application fails.
pub fn handle_apply() -> Result<()> {
    let mut config = Config::new();
    config.load_local_config();
    let ctx = ApplyContext::new(&config);
    
    let outcome = apply::run_apply(&ctx)?;
    apply::print_result(&outcome);
    Ok(())
}