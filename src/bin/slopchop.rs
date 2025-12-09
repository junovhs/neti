// src/bin/slopchop.rs
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

use slopchop_core::analysis::RuleEngine;
use slopchop_core::cli::{self, PackArgs};
use slopchop_core::config::Config;
use slopchop_core::discovery;
use slopchop_core::pack::OutputFormat;
use slopchop_core::project;
use slopchop_core::reporting;
use slopchop_core::roadmap_v2::{handle_command, RoadmapV2Command};
use slopchop_core::signatures::SignatureOptions;
use slopchop_core::tui::state::App;
use slopchop_core::wizard;

#[derive(Parser)]
#[command(name = "slopchop", version, about = "Code quality guardian")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(long)]
    ui: bool,
    #[arg(long)]
    init: bool,
}

#[derive(Subcommand)]
enum Commands {
    Prompt {
        #[arg(long, short)]
        copy: bool,
    },
    Check,
    Fix,
    Apply,
    Clean {
        #[arg(long, short)]
        commit: bool,
    },
    Config,
    Dashboard,
    #[command(subcommand)]
    Roadmap(RoadmapV2Command),
    /// Analyze codebase for consolidation opportunities
    Audit {
        /// Output format: terminal, json, or ai
        #[arg(long, default_value = "terminal")]
        format: String,
        /// Disable dead code detection
        #[arg(long)]
        no_dead: bool,
        /// Disable duplicate detection
        #[arg(long)]
        no_dups: bool,
        /// Disable pattern detection
        #[arg(long)]
        no_patterns: bool,
        /// Minimum lines for a code unit to be analyzed
        #[arg(long, default_value = "5")]
        min_lines: usize,
        /// Maximum number of opportunities to report
        #[arg(long, default_value = "50")]
        max: usize,
        /// Show verbose output
        #[arg(long, short)]
        verbose: bool,
    },
    Pack {
        #[arg(long, short)]
        stdout: bool,
        #[arg(long, short)]
        copy: bool,
        #[arg(long)]
        noprompt: bool,
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
        #[arg(long)]
        skeleton: bool,
        #[arg(long)]
        git_only: bool,
        #[arg(long)]
        no_git: bool,
        #[arg(long)]
        code_only: bool,
        #[arg(long, short)]
        verbose: bool,
        #[arg(long, value_name = "FILE")]
        target: Option<PathBuf>,
        #[arg(long, short, value_name = "FILE")]
        focus: Vec<PathBuf>,
        #[arg(long, default_value = "1")]
        depth: usize,
    },
    Trace {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        #[arg(long, short, default_value = "2")]
        depth: usize,
        #[arg(long, short, default_value = "4000")]
        budget: usize,
    },
    Map {
        #[arg(long, short)]
        deps: bool,
    },
    /// Generate a compact type signature map of the codebase
    Signatures {
        #[arg(long, short)]
        copy: bool,
        #[arg(long, short)]
        stdout: bool,
    },
}

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

        Commands::Apply | Commands::Prompt { .. } | Commands::Roadmap(_) => dispatch_tools(cmd),
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
        Commands::Apply => {
            cli::handle_apply()?;
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
            cli::handle_audit(
                format,
                *no_dead,
                *no_dups,
                *no_patterns,
                *min_lines,
                *max,
                *verbose,
            )?;
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
        eprintln!("{}", "✓ Created slopchop.toml".dimmed());
    }
}

