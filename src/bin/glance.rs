use anyhow::Result;
use clap::Parser;
use std::fs;
use std::io::{self, Write};

// Use the same core logic as Warden
use warden_core::config::{Config, GitMode};
use warden_core::enumerate::FileEnumerator;
use warden_core::filter::FileFilter;
use warden_core::heuristics::HeuristicFilter;

#[derive(Parser)]
#[command(name = "glance")]
#[command(about = "Context packer for LLMs. Dumps repo to stdout.")]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    /// Enable verbose logging (to stderr)
    #[arg(long, short)]
    verbose: bool,

    /// Force git-only mode
    #[arg(long)]
    git_only: bool,

    /// Force no-git mode
    #[arg(long)]
    no_git: bool,

    /// Only include code files
    #[arg(long)]
    code_only: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup identical to Warden
    let mut config = Config::new();
    config.verbose = cli.verbose;
    config.code_only = cli.code_only;

    if cli.git_only {
        config.git_mode = GitMode::Yes;
    } else if cli.no_git {
        config.git_mode = GitMode::No;
    }

    config.validate()?;

    // 1. Discovery
    if config.verbose {
        eprintln!("🔍 Enumerating files...");
    }
    // We clone config here so we can use it again later
    let enumerator = FileEnumerator::new(config.clone());
    let raw_files = enumerator.enumerate()?;

    // 2. Heuristics (remove binaries)
    let heuristic_filter = HeuristicFilter::new();
    let heuristics_files = heuristic_filter.filter(raw_files);

    // 3. Filtering (patterns/secrets)
    // FIX: We clone config here too, so 'config' is still alive for the next line
    let filter = FileFilter::new(config.clone())?;
    let target_files = filter.filter(heuristics_files);

    if config.verbose {
        eprintln!("📦 Packing {} files...", target_files.len());
    }

    // 4. Output Loop (The "Glance" Format)
    let mut stdout = io::stdout().lock();

    for path in target_files {
        let path_str = path.to_string_lossy();

        // Header
        writeln!(
            stdout,
            "================================================================================"
        )?;
        writeln!(stdout, "FILE: {path_str}")?;
        writeln!(
            stdout,
            "================================================================================"
        )?;

        // Content
        match fs::read_to_string(&path) {
            Ok(content) => {
                writeln!(stdout, "{content}")?;
            }
            Err(e) => {
                writeln!(stdout, "<ERROR READING FILE: {e}>")?;
            }
        }
        writeln!(stdout, "\n")?; // Extra spacing between files
    }

    if cli.verbose {
        eprintln!("✅ Done.");
    }
    Ok(())
}
