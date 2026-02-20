use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "neti", version, about = "AI Code Quality Guardian")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run structural checks on the codebase
    Check {
        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },

    /// Scan for violations
    Scan {
        #[arg(long, short)]
        verbose: bool,
        /// Run topology/locality analysis [EXPERIMENTAL]
        #[arg(long, short)]
        locality: bool,
        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },

    /// Create or reset the work branch
    Branch {
        #[arg(long, short)]
        force: bool,
    },

    /// Promote work branch to main
    Promote {
        #[arg(long)]
        dry_run: bool,
    },

    /// Abort work branch and return to main
    Abort,

    /// Clean up artifacts
    Clean {
        #[arg(long, short)]
        commit: bool,
    },

    /// Interactive configuration editor
    Config,

    /// Run mutation testing to find test gaps [EXPERIMENTAL]
    Mutate {
        /// Number of parallel workers (reserved for future use)
        #[arg(long, short)]
        workers: Option<usize>,
        /// Test timeout in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,
        /// Output results as JSON
        #[arg(long)]
        json: bool,
        /// Filter files by path pattern
        #[arg(long, short)]
        filter: Option<String>,
    },
}
