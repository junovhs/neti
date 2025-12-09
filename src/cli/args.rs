use clap::{Parser, Subcommand};
use std::path::PathBuf;
use crate::pack::OutputFormat;
use crate::roadmap_v2::RoadmapV2Command;

#[derive(Parser)]
#[command(name = "slopchop", version, about = "Code quality guardian")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    #[arg(long)]
    pub ui: bool,
    #[arg(long)]
    pub init: bool,
}

#[derive(Subcommand)]
pub enum Commands {
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
