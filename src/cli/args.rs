// src/cli/args.rs
use crate::pack::OutputFormat;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "slopchop", version, about = "AI Code Quality Guardian")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run structural checks on the codebase
    Check,

    /// Scan for violations
    Scan {
        #[arg(long, short)]
        verbose: bool,
        /// Run topology/locality analysis [EXPERIMENTAL]
        #[arg(long, short)]
        locality: bool,
    },

    /// Apply AI-generated changes to staging area
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
        #[arg(long)]
        sanitize: bool,
        #[arg(long, conflicts_with = "sanitize")]
        strict: bool,
    },

    /// Clean up artifacts
    Clean {
        #[arg(long, short)]
        commit: bool,
    },

    /// Show or edit configuration
    Config,

    /// Find code duplication and consolidation opportunities [EXPERIMENTAL]
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

    /// Generate AI context from codebase
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
        #[arg(long, short, value_name = "FILE", num_args = 1..)]
        focus: Vec<PathBuf>,
        #[arg(long, default_value = "1")]
        depth: usize,
    },

    /// Show repository structure [EXPERIMENTAL]
    Map {
        /// Show dependencies for each file
        #[arg(long, short)]
        deps: bool,
    },

    /// Generate type-surface signatures [EXPERIMENTAL]
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
    pub sanitize: bool,
    pub strict: bool,
}
