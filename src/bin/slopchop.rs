// src/bin/slopchop.rs
use std::fs;
use std::io;
use std::path::Path;
use std::process;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use slopchop_core::analysis::RuleEngine;
use slopchop_core::cli::args::ApplyArgs;
use slopchop_core::cli::{self, Cli, Commands, PackArgs};
use slopchop_core::config::Config;
use slopchop_core::discovery;
use slopchop_core::project;
use slopchop_core::reporting;
use slopchop_core::signatures::SignatureOptions;
use slopchop_core::tui::runner;
use slopchop_core::tui::state::App;

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {e}", "error:".red().bold());
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    ensure_config_exists();
    dispatch(&cli)
}

fn dispatch(cli: &Cli) -> Result<()> {
    match &cli.command {
        Some(cmd) => dispatch_command(cmd),
        None if cli.ui => run_tui(),
        None => run_scan(),
    }
}

fn dispatch_command(cmd: &Commands) -> Result<()> {
    match cmd {
        Commands::Pack { .. }
        | Commands::Trace { .. }
        | Commands::Map { .. }
        | Commands::Signatures { .. }
        | Commands::Audit { .. } => dispatch_analysis(cmd),
        Commands::Check
        | Commands::Fix
        | Commands::Clean { .. }
        | Commands::Config
        | Commands::Dashboard => dispatch_maintenance(cmd),
        Commands::Apply { .. } | Commands::Prompt { .. } => dispatch_tools(cmd),
    }
}

fn dispatch_maintenance(cmd: &Commands) -> Result<()> {
    match cmd {
        Commands::Check => cli::handle_check(),
        Commands::Fix => cli::handle_fix(),
        Commands::Config => slopchop_core::tui::run_config(),
        Commands::Dashboard => cli::handle_dashboard(),
        Commands::Clean { commit } => slopchop_core::clean::run(*commit),
        _ => unreachable!(),
    }
}

fn dispatch_tools(cmd: &Commands) -> Result<()> {
    match cmd {
        Commands::Apply {
            force,
            dry_run,
            stdin,
            check,
            file,
            reset,
            promote,
        } => {
            let args = ApplyArgs {
                force: *force,
                dry_run: *dry_run,
                stdin: *stdin,
                check: *check,
                file: file.clone(),
                reset: *reset,
                promote: *promote,
            };
            cli::handle_apply(&args)
        }
        Commands::Prompt { copy } => cli::handle_prompt(*copy),
        _ => unreachable!(),
    }
}

fn dispatch_analysis(cmd: &Commands) -> Result<()> {
    match cmd {
        Commands::Trace {
            file,
            depth,
            budget,
        } => cli::handle_trace(file, *depth, *budget),
        Commands::Map { deps } => cli::handle_map(*deps),
        Commands::Pack { .. } => dispatch_pack(cmd),
        Commands::Signatures { copy, stdout } => cli::handle_signatures(SignatureOptions {
            copy: *copy,
            stdout: *stdout,
        }),
        Commands::Audit {
            format,
            no_dead,
            no_dups,
            no_patterns,
            min_lines,
            max,
            verbose,
        } => cli::handle_audit(&cli::audit::AuditCliOptions {
            format,
            no_dead: *no_dead,
            no_dups: *no_dups,
            no_patterns: *no_patterns,
            min_lines: *min_lines,
            max: *max,
            verbose: *verbose,
        }),
        _ => unreachable!(),
    }
}

fn dispatch_pack(cmd: &Commands) -> Result<()> {
    if let Commands::Pack {
        stdout,
        copy,
        noprompt,
        format,
        skeleton,
        code_only,
        verbose,
        target,
        focus,
        depth,
    } = cmd
    {
        cli::handle_pack(PackArgs {
            stdout: *stdout,
            copy: *copy,
            noprompt: *noprompt,
            format: format.clone(),
            skeleton: *skeleton,
            code_only: *code_only,
            verbose: *verbose,
            target: target.clone(),
            focus: focus.clone(),
            depth: *depth,
        })?;
    }
    Ok(())
}

fn run_scan() -> Result<()> {
    let config = Config::load();
    let report = RuleEngine::new(config.clone()).scan(discovery::discover(&config)?);
    reporting::print_report(&report)?;
    if report.has_errors() {
        process::exit(1);
    }
    Ok(())
}

fn run_tui() -> Result<()> {
    use ratatui::{backend::CrosstermBackend, Terminal};
    let config = Config::load();
    let report = RuleEngine::new(config.clone()).scan(discovery::discover(&config)?);

    runner::setup_terminal()?;
    let mut term = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let mut app = App::new(report);
    let _ = app.run(&mut term);
    runner::restore_terminal()?;
    Ok(())
}

fn ensure_config_exists() {
    if Path::new("slopchop.toml").exists() {
        return;
    }
    let content = project::generate_toml(
        project::ProjectType::detect(),
        project::Strictness::Standard,
    );
    let _ = fs::write("slopchop.toml", content);
}
