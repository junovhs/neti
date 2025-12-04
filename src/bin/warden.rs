// src/bin/warden.rs
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

use warden_core::analysis::RuleEngine;
use warden_core::cli::{self, PackArgs};
use warden_core::config::Config;
use warden_core::discovery;
use warden_core::pack::OutputFormat;
use warden_core::project;
use warden_core::reporting;
use warden_core::roadmap::cli::{handle_command, RoadmapCommand};
use warden_core::tui::state::App;
use warden_core::wizard;

#[derive(Parser)]
#[command(name = "warden", version, about = "Code quality guardian")]
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
    Prompt { #[arg(long, short)] copy: bool },
    Check,
    Fix,
    Apply,
    Clean { #[arg(long, short)] commit: bool },
    Config,
    #[command(subcommand)]
    Roadmap(RoadmapCommand),
    Pack {
        #[arg(long, short)] stdout: bool,
        #[arg(long, short)] copy: bool,
        #[arg(long)] noprompt: bool,
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)] format: OutputFormat,
        #[arg(long)] skeleton: bool,
        #[arg(long)] git_only: bool,
        #[arg(long)] no_git: bool,
        #[arg(long)] code_only: bool,
        #[arg(long, short)] verbose: bool,
        #[arg(long, value_name = "FILE")] target: Option<PathBuf>,
        #[arg(long, short, value_name = "FILE")] focus: Vec<PathBuf>,
        #[arg(long, default_value = "1")] depth: usize,
    },
    Trace {
        #[arg(value_name = "FILE")] file: PathBuf,
        #[arg(long, short, default_value = "2")] depth: usize,
        #[arg(long, short, default_value = "4000")] budget: usize,
    },
    Map { #[arg(long, short)] deps: bool },
    Context {
        #[arg(long, short)] verbose: bool,
        #[arg(long, short)] copy: bool,
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
        return wizard::run();
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
        | Commands::Context { .. } => dispatch_analysis(cmd),

        Commands::Check
        | Commands::Fix
        | Commands::Apply
        | Commands::Clean { .. } => dispatch_action(cmd),

        Commands::Config | Commands::Prompt { .. } | Commands::Roadmap(_) => dispatch_util(cmd),
    }
}

fn dispatch_action(cmd: &Commands) -> Result<()> {
    match cmd {
        Commands::Check => { cli::handle_check(); Ok(()) }
        Commands::Fix => { cli::handle_fix(); Ok(()) }
        Commands::Apply => cli::handle_apply(),
        Commands::Clean { commit } => warden_core::clean::run(*commit),
        _ => unreachable!(),
    }
}

fn dispatch_util(cmd: &Commands) -> Result<()> {
    match cmd {
        Commands::Config => warden_core::tui::run_config(),
        Commands::Prompt { copy } => cli::handle_prompt(*copy),
        Commands::Roadmap(sub) => handle_command(sub.clone()),
        _ => unreachable!(),
    }
}

fn dispatch_analysis(cmd: &Commands) -> Result<()> {
    match cmd {
        Commands::Trace { file, depth, budget } => cli::handle_trace(file, *depth, *budget),
        Commands::Map { deps } => cli::handle_map(*deps),
        Commands::Context { verbose, copy } => cli::handle_context(*verbose, *copy),
        Commands::Pack { .. } => dispatch_pack(cmd),
        _ => unreachable!(),
    }
}

fn dispatch_pack(cmd: &Commands) -> Result<()> {
    if let Commands::Pack { stdout, copy, noprompt, format, skeleton, git_only, no_git,
        code_only, verbose, target, focus, depth } = cmd {
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
        })
    } else {
        Ok(())
    }
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

    let res = App::new(report).run(&mut term);

    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    term.show_cursor()?;
    res
}

fn load_config() -> Config {
    let mut c = Config::new();
    c.load_local_config();
    c
}

fn ensure_config_exists() {
    if Path::new("warden.toml").exists() {
        return;
    }
    let proj = project::ProjectType::detect();
    let content = project::generate_toml(proj, project::Strictness::Standard);
    if fs::write("warden.toml", &content).is_ok() {
        eprintln!("{}", "✨ Created warden.toml".dimmed());
    }
}