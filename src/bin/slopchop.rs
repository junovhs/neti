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
use slopchop_core::roadmap_v2::handle_command;
use slopchop_core::signatures::SignatureOptions;
use slopchop_core::tui::state::App;
use slopchop_core::wizard;

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {e}", "error:".red().bold());
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    if cli.init {
        wizard::run()?;
        return Ok(());
    }
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

        Commands::Apply { .. } | Commands::Prompt { .. } | Commands::Roadmap(_) => {
            dispatch_tools(cmd)
        }
    }
}

fn dispatch_maintenance(cmd: &Commands) -> Result<()> {
    match cmd {
        Commands::Check => {
            cli::handle_check()?;
            Ok(())
        }
        Commands::Fix => {
            cli::handle_fix()?;
            Ok(())
        }
        Commands::Config => {
            slopchop_core::tui::run_config()?;
            Ok(())
        }
        Commands::Dashboard => {
            cli::handle_dashboard()?;
            Ok(())
        }
        Commands::Clean { commit } => {
            slopchop_core::clean::run(*commit)?;
            Ok(())
        }
        _ => unreachable!(),
    }
}

fn dispatch_tools(cmd: &Commands) -> Result<()> {
    match cmd {
        Commands::Apply {
            force,
            dry_run,
            stdin,
            file,
            no_commit,
            no_push,
        } => {
            let args = ApplyArgs {
                force: *force,
                dry_run: *dry_run,
                stdin: *stdin,
                file: file.clone(),
                no_commit: *no_commit,
                no_push: *no_push,
            };
            cli::handle_apply(&args)?;
            Ok(())
        }
        Commands::Prompt { copy } => {
            cli::handle_prompt(*copy)?;
            Ok(())
        }
        Commands::Roadmap(sub) => {
            handle_command(sub.clone())?;
            Ok(())
        }
        _ => unreachable!(),
    }
}

fn dispatch_analysis(cmd: &Commands) -> Result<()> {
    match cmd {
        Commands::Trace {
            file,
            depth,
            budget,
        } => {
            cli::handle_trace(file, *depth, *budget)?;
            Ok(())
        }
        Commands::Map { deps } => {
            cli::handle_map(*deps)?;
            Ok(())
        }
        Commands::Pack { .. } => dispatch_pack(cmd),
        Commands::Signatures { copy, stdout } => {
            cli::handle_signatures(SignatureOptions {
                copy: *copy,
                stdout: *stdout,
            })?;
            Ok(())
        }
        Commands::Audit {
            format,
            no_dead,
            no_dups,
            no_patterns,
            min_lines,
            max,
            verbose,
        } => {
            cli::handle_audit(&cli::audit::AuditCliOptions {
                format,
                no_dead: *no_dead,
                no_dups: *no_dups,
                no_patterns: *no_patterns,
                min_lines: *min_lines,
                max: *max,
                verbose: *verbose,
            })?;
            Ok(())
        }
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
        git_only,
        no_git,
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
            git_only: *git_only,
            no_git: *no_git,
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
    let config = load_config();
    let report = RuleEngine::new(config.clone()).scan(discovery::discover(&config)?);
    reporting::print_report(&report)?;
    if report.has_errors() {
        process::exit(1);
    }
    Ok(())
}

fn run_tui() -> Result<()> {
    use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
    use crossterm::execute;
    use crossterm::terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    };
    use ratatui::backend::CrosstermBackend;
    use ratatui::Terminal;

    let config = load_config();
    let report = RuleEngine::new(config.clone()).scan(discovery::discover(&config)?);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let mut term = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut app = App::new(report);
    let _ = app.run(&mut term);

    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;
    Ok(())
}

fn load_config() -> Config {
    let mut c = Config::new();
    c.load_local_config();
    c
}

fn ensure_config_exists() {
    if Path::new("slopchop.toml").exists() {
        return;
    }
    let proj = project::ProjectType::detect();
    let content = project::generate_toml(proj, project::Strictness::Standard);
    if fs::write("slopchop.toml", &content).is_ok() {
        eprintln!("{}", "� Created slopchop.toml".dimmed());
    }
}