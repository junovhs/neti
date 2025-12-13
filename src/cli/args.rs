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
    /// Apply AI-generated code changes from clipboard, stdin, or file
    Apply {
        /// Skip interactive confirmation
        #[arg(long, short)]
        force: bool,
        /// Validate without writing files
        #[arg(long)]
        dry_run: bool,
        /// Read input from stdin instead of clipboard
        #[arg(long)]
        stdin: bool,
        /// Read input from file instead of clipboard
        #[arg(long, value_name = "FILE")]
        file: Option<PathBuf>,
        /// Skip git commit even if `auto_commit` is enabled
        #[arg(long)]
        no_commit: bool,
        /// Skip git push even if `auto_push` is enabled
        #[arg(long)]
        no_push: bool,
    },
    Clean {
        #[arg(long, short)]
        commit: bool,
    },
    Config,
    Dashboard,
    #[command(subcommand)]
    Roadmap(RoadmapV2Command),
    Audit {
        #[arg(long, default_value = "terminal")]
        format: String,
        #[arg(long)]
        no_dead: bool,
        #[arg(long)]
        no_dups: bool,
        #[arg(long)]
        no_patterns: bool,
        #[arg(long, default_value = "5")]
        min_lines: usize,
        #[arg(long, default_value = "50")]
        max: usize,
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
    Signatures {
        #[arg(long, short)]
        copy: bool,
        #[arg(long, short)]
        stdout: bool,
    },
}

/// Arguments for the Apply command (used by handlers)
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Default)]
pub struct ApplyArgs {
    pub force: bool,
    pub dry_run: bool,
    pub stdin: bool,
    pub file: Option<PathBuf>,
    pub no_commit: bool,
    pub no_push: bool,
}