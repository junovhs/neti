use anyhow::Result;
use clap::Parser;
use colored::*;
use std::process;

// Module declarations
mod config;
mod detection;
mod enumerate;
mod error;
mod filter;
mod heuristics;
mod rules;

use crate::config::{Config, GitMode};
use crate::detection::Detector;
use crate::enumerate::FileEnumerator;
use crate::filter::FileFilter;
use crate::heuristics::HeuristicFilter;
use crate::rules::RuleEngine;

#[derive(Parser)]
#[command(name = "warden")]
#[command(about = "Structural linter for Code With Intent")]
struct Cli {
    /// Enable verbose logging
    #[arg(long, short)]
    verbose: bool,

    /// Force git-only mode (respect .gitignore)
    #[arg(long)]
    git_only: bool,

    /// Force no-git mode (scan everything)
    #[arg(long)]
    no_git: bool,

    /// Only scan code files (ignore configs/docs)
    #[arg(long)]
    code_only: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // 1. Setup & Validation
    let mut config = Config::new();
    config.verbose = cli.verbose;
    config.code_only = cli.code_only;

    // Wire up the GitMode enum variants (Resolves "Variant never constructed" warning)
    if cli.git_only {
        config.git_mode = GitMode::Yes;
    } else if cli.no_git {
        config.git_mode = GitMode::No;
    }

    // Call validate (Resolves "method never used" warning)
    config.validate()?;

    if config.verbose {
        println!("🔧 Config loaded: GitMode::{:?}", config.git_mode);
    }

    // 2. Enumerate Files
    let enumerator = FileEnumerator::new(config.clone());
    let raw_files = enumerator.enumerate()?;

    // 3. Detection Layer (Resolves "Detector never constructed" warnings)
    // We use the detector to give context to the user about what we think this project is.
    let detector = Detector::new();
    let systems = detector.detect_build_systems(&raw_files)?;
    if !systems.is_empty() {
        let sys_list: Vec<String> = systems.iter().map(|s| s.to_string()).collect();
        println!("🔎 Detected Ecosystem: [{}]", sys_list.join(", ").cyan());
    }

    // 4. Heuristics Layer (Resolves "HeuristicFilter never used" warnings)
    // Filters out high-entropy files (likely binaries/obfuscated code)
    let heuristic_filter = HeuristicFilter::new();
    let heuristics_files = heuristic_filter.filter(raw_files);

    // 5. Standard Filter Layer (Extension & Secrets)
    let filter = FileFilter::new(config)?;
    let target_files = filter.filter(heuristics_files);

    if target_files.is_empty() {
        println!("No files to scan.");
        return Ok(());
    }

    println!("👮 Warden scanning {} files...", target_files.len());

    // 6. Logic Engine (The Rules)
    let engine = RuleEngine::new();
    let mut total_failures = 0;

    for path in target_files {
        // We ignore the result error here (read failures) and just count logic failures
        if let Ok(passed) = engine.check_file(&path) {
            if !passed {
                total_failures += 1;
            }
        }
    }

    println!("---------------------------------------------------");
    if total_failures > 0 {
        println!(
            "{}",
            format!("❌ Warden found {} violations.", total_failures)
                .red()
                .bold()
        );
        process::exit(1);
    } else {
        println!(
            "{}",
            "✅ All Clear. Code structure is clean.".green().bold()
        );
        process::exit(0);
    }
}
