// src/cli/handlers.rs
//! Command handlers for the slopchop CLI.

use std::path::Path;

use anyhow::Result;
use colored::Colorize;

use crate::apply::{self, types::ApplyContext};
use crate::config::Config;
use crate::context::{self, ContextOptions};
use crate::pack::{self, OutputFormat, PackOptions};
use crate::prompt::PromptGenerator;
use crate::trace::{self, TraceOptions};

/// Runs the check pipeline.
pub fn handle_check() {
    run_pipeline("check");
}

/// Runs the fix pipeline.
pub fn handle_fix() {
    run_pipeline("fix");
}

/// Displays the repository map.
///
/// # Errors
/// Returns error if trace fails.
pub fn handle_map(show_deps: bool) -> Result<()> {
    println!("{}", trace::map(show_deps)?);
    Ok(())
}

/// Traces dependencies from a file.
///
/// # Errors
/// Returns error if trace fails.
pub fn handle_trace(file: &Path, depth: usize, budget: usize) -> Result<()> {
    let opts = TraceOptions {
        anchor: file.to_path_buf(),
        depth,
        budget,
    };
    println!("{}", trace::run(&opts)?);
    Ok(())
}

/// Generates and displays context map.
///
/// # Errors
/// Returns error if context generation or clipboard fails.
pub fn handle_context(verbose: bool, copy: bool) -> Result<()> {
    let opts = ContextOptions { verbose };
    let output = context::run(&opts)?;

    if copy {
        crate::clipboard::copy_to_clipboard(&output)?;
        println!("{}", "✓ Context map copied to clipboard".green());
    } else {
        println!("{output}");
    }

    Ok(())
}

/// Generates and optionally copies the prompt.
///
/// # Errors
/// Returns error if prompt generation or clipboard fails.
pub fn handle_prompt(copy: bool) -> Result<()> {
    let mut config = Config::new();
    config.load_local_config();
    let prompt = PromptGenerator::new(config.rules.clone()).generate()?;

    if copy {
        crate::clipboard::copy_to_clipboard(&prompt)?;
        println!("{}", "✓ Copied to clipboard".green());
    } else {
        println!("{prompt}");
    }
    Ok(())
}

/// Applies changes from clipboard.
///
/// # Errors
/// Returns error if apply fails.
pub fn handle_apply() -> Result<()> {
    let mut config = Config::new();
    config.load_local_config();
    let outcome = apply::run_apply(&ApplyContext::new(&config))?;
    apply::print_result(&outcome);
    Ok(())
}

/// Pack command arguments.
#[allow(clippy::struct_excessive_bools)]
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
    pub target: Option<std::path::PathBuf>,
    pub focus: Vec<std::path::PathBuf>,
    pub depth: usize,
}

/// Runs the pack command.
///
/// # Errors
/// Returns error if packing fails.
pub fn handle_pack(args: PackArgs) -> Result<()> {
    pack::run(&PackOptions {
        stdout: args.stdout,
        copy: args.copy,
        prompt: !args.noprompt,
        format: args.format,
        skeleton: args.skeleton,
        git_only: args.git_only,
        no_git: args.no_git,
        code_only: args.code_only,
        verbose: args.verbose,
        target: args.target,
        focus: args.focus,
        depth: args.depth,
    })
}

fn run_pipeline(name: &str) {
    use std::process;

    let mut config = Config::new();
    config.load_local_config();

    let Some(commands) = config.commands.get(name) else {
        eprintln!("{} No '{}' command configured", "error:".red(), name);
        process::exit(1);
    };

    println!("{} Running '{}' pipeline...", "✨".green(), name);
    for cmd in commands {
        exec_cmd(cmd);
    }
}

fn exec_cmd(cmd: &str) {
    use std::io;
    use std::process::{self, Command};

    println!("   {} {}", "exec:".dimmed(), cmd.dimmed());
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let (prog, args) = parts.split_first().unwrap_or((&"", &[]));

    match Command::new(prog).args(args).status() {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!("{} Exit code {}", "✗".red(), s.code().unwrap_or(1));
            process::exit(s.code().unwrap_or(1));
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            eprintln!("{} Not found: {prog}", "error:".red());
            process::exit(1);
        }
        Err(e) => {
            eprintln!("{} {e}", "error:".red());
            process::exit(1);
        }
    }
}
