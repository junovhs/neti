use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::fs;
use std::io;
use std::process;

use warden_core::config::{Config, GitMode};
use warden_core::enumerate::FileEnumerator;
use warden_core::filter::FileFilter;
use warden_core::heuristics::HeuristicFilter;
use warden_core::rules::RuleEngine;
use warden_core::tui::state::App;
use warden_core::types::ScanReport;

const DEFAULT_TOML: &str = r#"# warden.toml
[rules]
max_file_tokens = 2000
max_cyclomatic_complexity = 10
max_nesting_depth = 4
max_function_args = 5
max_function_words = 3
ignore_naming_on = ["tests", "spec"]
"#;

#[derive(Parser)]
#[command(name = "warden")]
#[command(about = "Structural linter for Code With Intent")]
#[allow(clippy::struct_excessive_bools)] // Standard for Clap CLI structs
struct Cli {
    #[arg(long, short)]
    verbose: bool,
    #[arg(long)]
    git_only: bool,
    #[arg(long)]
    no_git: bool,
    #[arg(long)]
    code_only: bool,
    #[arg(long)]
    init: bool,
    /// Launch the TUI dashboard
    #[arg(long)]
    ui: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.init {
        return handle_init();
    }

    let config = initialize_config(&cli)?;
    let target_files = run_scan_discovery(&config)?;

    if target_files.is_empty() {
        println!("No files to scan.");
        return Ok(());
    }

    // Run Analysis
    let engine = RuleEngine::new(config);
    if cli.verbose {
        println!("👮 Analyzing {} files...", target_files.len());
    }
    let report = engine.scan(target_files);

    if cli.ui {
        run_tui(report)?;
    } else {
        print_report(&report);
        if report.total_violations > 0 {
            process::exit(1);
        }
    }

    Ok(())
}

fn run_tui(report: ScanReport) -> Result<()> {
    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Run App
    let mut app = App::new(report);
    let res = app.run(&mut terminal);

    // Restore Terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }
    Ok(())
}

fn print_report(report: &ScanReport) {
    let mut failures = 0;
    for file in &report.files {
        if !file.is_clean() {
            failures += file.violations.len();
            for v in &file.violations {
                let filename = file.path.to_string_lossy();
                let line_num = v.row + 1;
                println!("{}: {}", "error".red().bold(), v.message.bold());
                println!("  {} {}:{}:1", "-->".blue(), filename, line_num);
                println!("   {}", "|".blue());
                println!(
                    "   {} {}: Action required",
                    "=".blue().bold(),
                    v.law.white().bold()
                );
                println!();
            }
        }
    }

    if failures > 0 {
        println!(
            "{}",
            format!(
                "❌ Warden found {failures} violations in {}ms.",
                report.duration_ms
            )
            .red()
            .bold()
        );
    } else {
        println!(
            "{}",
            format!(
                "✅ All Clear. Scanned {} tokens in {}ms.",
                report.total_tokens, report.duration_ms
            )
            .green()
            .bold()
        );
    }
}

fn handle_init() -> Result<()> {
    if std::path::Path::new("warden.toml").exists() {
        println!("{}", "⚠️ warden.toml already exists.".yellow());
    } else {
        fs::write("warden.toml", DEFAULT_TOML)?;
        println!("{}", "✅ Created warden.toml".green());
    }
    Ok(())
}

fn initialize_config(cli: &Cli) -> Result<Config> {
    let mut config = Config::new();
    config.verbose = cli.verbose;
    config.code_only = cli.code_only;

    if cli.git_only {
        config.git_mode = GitMode::Yes;
    } else if cli.no_git {
        config.git_mode = GitMode::No;
    }

    config.load_local_config();
    config.validate()?;
    Ok(config)
}

fn run_scan_discovery(config: &Config) -> Result<Vec<std::path::PathBuf>> {
    let enumerator = FileEnumerator::new(config.clone());
    let raw_files = enumerator.enumerate()?;

    // Optional: Detect ecosystem
    let detector = warden_core::detection::Detector::new();
    if let Ok(systems) = detector.detect_build_systems(&raw_files) {
        if !systems.is_empty() && config.verbose {
            let sys_list: Vec<String> = systems.iter().map(ToString::to_string).collect();
            println!("🔎 Detected Ecosystem: [{}]", sys_list.join(", ").cyan());
        }
    }

    let heuristic_filter = HeuristicFilter::new();
    let heuristics_files = heuristic_filter.filter(raw_files);

    let filter = FileFilter::new(config.clone())?;
    Ok(filter.filter(heuristics_files))
}
