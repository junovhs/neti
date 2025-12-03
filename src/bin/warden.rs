// src/bin/warden.rs
use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

use warden_core::analysis::RuleEngine;
use warden_core::apply::{self, types::ApplyContext};
use warden_core::config::Config;
use warden_core::discovery;
use warden_core::pack::{self, OutputFormat, PackOptions};
use warden_core::prompt::PromptGenerator;
use warden_core::reporting;
use warden_core::roadmap::cli::{handle_command, RoadmapCommand};
use warden_core::trace::{self, TraceOptions};
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
    #[command(subcommand)]
    Roadmap(RoadmapCommand),
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
        #[arg(value_name = "TARGET")]
        target: Option<PathBuf>,
    },
    Trace {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        #[arg(long, short, default_value = "2")]
        depth: usize,
        #[arg(long, short, default_value = "4000")]
        budget: usize,
    },
    Map,
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
        None => dispatch_default(cli.ui),
    }
}

fn dispatch_default(ui: bool) -> Result<()> {
    if ui {
        run_tui()
    } else {
        run_scan()
    }
}

fn dispatch_command(cmd: &Commands) -> Result<()> {
    if let Some(result) = try_simple_commands(cmd) {
        return result;
    }
    dispatch_complex_commands(cmd)
}

fn try_simple_commands(cmd: &Commands) -> Option<Result<()>> {
    match cmd {
        Commands::Check => Some(handle_check()),
        Commands::Fix => Some(handle_fix()),
        Commands::Config => Some(warden_core::tui::run_config()),
        Commands::Apply => Some(handle_apply()),
        Commands::Map => Some(handle_map()),
        _ => None,
    }
}

fn dispatch_complex_commands(cmd: &Commands) -> Result<()> {
    match cmd {
        Commands::Prompt { copy } => handle_prompt(*copy),
        Commands::Clean { commit } => warden_core::clean::run(*commit),
        Commands::Roadmap(sub) => handle_command(sub.clone()),
        Commands::Pack { .. } => handle_pack(cmd),
        Commands::Trace { file, depth, budget } => handle_trace(file, *depth, *budget),
        _ => Ok(()),
    }
}

#[allow(clippy::unnecessary_wraps)]
fn handle_check() -> Result<()> {
    run_pipeline("check");
    Ok(())
}

#[allow(clippy::unnecessary_wraps)]
fn handle_fix() -> Result<()> {
    run_pipeline("fix");
    Ok(())
}

fn handle_map() -> Result<()> {
    println!("{}", trace::map()?);
    Ok(())
}

fn handle_trace(file: &Path, depth: usize, budget: usize) -> Result<()> {
    let opts = TraceOptions {
        anchor: file.to_path_buf(),
        depth,
        budget,
    };
    println!("{}", trace::run(&opts)?);
    Ok(())
}

fn handle_pack(cmd: &Commands) -> Result<()> {
    let Commands::Pack {
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
    } = cmd
    else {
        return Ok(());
    };
    pack::run(&PackOptions {
        stdout: *stdout,
        copy: *copy,
        prompt: !*noprompt,
        format: format.clone(),
        skeleton: *skeleton,
        git_only: *git_only,
        no_git: *no_git,
        code_only: *code_only,
        verbose: *verbose,
        target: target.clone(),
    })
}

fn handle_apply() -> Result<()> {
    let mut config = Config::new();
    config.load_local_config();
    let outcome = apply::run_apply(&ApplyContext::new(&config))?;
    apply::print_result(&outcome);
    Ok(())
}

fn ensure_config_exists() {
    if Path::new("warden.toml").exists() {
        return;
    }
    let project = warden_core::project::ProjectType::detect();
    let content =
        warden_core::project::generate_toml(project, warden_core::project::Strictness::Standard);
    if fs::write("warden.toml", &content).is_ok() {
        eprintln!("{}", "?? Created warden.toml".dimmed());
    }
}

fn handle_prompt(copy: bool) -> Result<()> {
    let mut config = Config::new();
    config.load_local_config();
    let prompt = PromptGenerator::new(config.rules.clone()).generate()?;
    if copy {
        warden_core::clipboard::copy_to_clipboard(&prompt)?;
        println!("{}", "� Copied to clipboard".green());
    } else {
        println!("{prompt}");
    }
    Ok(())
}

fn run_pipeline(name: &str) {
    let mut config = Config::new();
    config.load_local_config();
    let Some(commands) = config.commands.get(name) else {
        eprintln!("{} No '{}' command configured", "error:".red(), name);
        process::exit(1);
    };
    println!("{} Running '{}' pipeline...", "??".green(), name);
    for cmd in commands {
        exec_cmd(cmd);
    }
}

fn exec_cmd(cmd: &str) {
    println!("   {} {}", "exec:".dimmed(), cmd.dimmed());
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let (prog, args) = parts.split_first().unwrap_or((&"", &[]));
    match Command::new(prog).args(args).status() {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!("{} Exit code {}", "?".red(), s.code().unwrap_or(1));
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
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;
    res
}

fn load_config() -> Config {
    let mut c = Config::new();
    c.load_local_config();
    c
}