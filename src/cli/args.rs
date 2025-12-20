// src/cli/args.rs
use crate::pack::OutputFormat;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "slopchop", version, about = "AI Code Quality Guardian")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    #[arg(long)]
    pub ui: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Prompt {
        #[arg(long, short)]
        copy: bool,
    },
    Check,
    Fix,
    Apply {
        #[arg(long, short)]
        force: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        stdin: bool,
        #[arg(long, short)]
        check: bool,
        #[arg(long, value_name = "FILE")]
        file: Option<PathBuf>,
        #[arg(long)]
        reset: bool,
        #[arg(long)]
        promote: bool,
    },
    Clean {
        #[arg(long, short)]
        commit: bool,
    },
    Config,
    Dashboard,
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

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Default)]
pub struct ApplyArgs {
    pub force: bool,
    pub dry_run: bool,
    pub stdin: bool,
    pub check: bool,
    pub file: Option<PathBuf>,
    pub reset: bool,
    pub promote: bool,
}
