// src/mutate/mod.rs
//! Cross-language mutation testing [EXPERIMENTAL].
//!
//! This module provides mutation testing capabilities across Rust, TypeScript,
//! and Python codebases. It discovers mutable points in source code, applies
//! mutations, runs tests, and reports which mutations survive (test gaps).
//!
//! # Architecture
//!
//! - `discovery`: Finds mutation points using tree-sitter AST analysis
//! - `mutations`: Defines mutation types and application logic
//! - `runner`: Executes tests against mutated code (serial, v1)
//! - `report`: Formats results for terminal and JSON output
//!
//! # Example
//!
//! ```ignore
//! neti mutate --filter src/tokens.rs --timeout 30
//! ```

pub mod discovery;
pub mod mutations;
pub mod report;
pub mod runner;

use crate::config::Config;
use crate::discovery::discover;
use crate::project::ProjectType;
use anyhow::Result;
use colored::Colorize;
use runner::{MutationSummary, RunnerConfig};
use std::path::{Path, PathBuf};

/// Options for mutation testing.
#[derive(Debug, Clone)]
pub struct MutateOptions {
    pub workers: Option<usize>,
    pub timeout_secs: u64,
    pub json: bool,
    pub filter: Option<String>,
}

impl Default for MutateOptions {
    fn default() -> Self {
        Self {
            workers: None,
            timeout_secs: 30,
            json: false,
            filter: None,
        }
    }
}

/// Result of a mutation testing run.
#[derive(Debug)]
pub struct MutateReport {
    pub summary: MutationSummary,
    pub results: Vec<runner::MutationResult>,
}

/// Runs mutation testing on the codebase.
///
/// # Errors
/// Returns error if discovery or test execution fails.
pub fn run(workdir: &Path, opts: &MutateOptions) -> Result<MutateReport> {
    let config = Config::load();

    // Discover source files
    let files = discover(&config)?;

    // Detect project type for test command
    let project_type = crate::project::ProjectType::detect_in(workdir);
    let mut runner_config = config_for_project(project_type);

    // Apply options
    if let Some(workers) = opts.workers {
        runner_config.workers = workers;
    }
    runner_config.timeout_secs = opts.timeout_secs;

    // Filter files if specified
    let target_files = filter_files(&files, opts.filter.as_deref());

    if !opts.json {
        print_header(&target_files, &runner_config);
    }

    // Discover mutation points
    let points = discover_all_mutations(&target_files);

    if points.is_empty() {
        return Ok(MutateReport {
            summary: MutationSummary {
                total: 0,
                killed: 0,
                survived: 0,
                score: 100.0,
                total_duration_ms: 0,
            },
            results: Vec::new(),
        });
    }

    if !opts.json {
        println!(
            "Found {} mutation points across {} files\n",
            points.len().to_string().cyan(),
            target_files.len()
        );
    }

    // Run mutations with progress reporting
    let results = runner::run_mutations(&points, &runner_config, workdir, |cur, total, result| {
        if !opts.json {
            println!("{}", report::format_progress(cur, total, result));
        }
    })?;

    let summary = runner::summarize(&results);

    // Print final report
    if opts.json {
        println!("{}", report::format_json(&results, &summary));
    } else {
        println!("{}", report::format_summary(&summary));
        println!("{}", report::format_survivors(&results));
    }

    Ok(MutateReport { summary, results })
}

/// Returns appropriate runner config for the project type.
fn config_for_project(project_type: ProjectType) -> RunnerConfig {
    match project_type {
        ProjectType::Rust => RunnerConfig::rust(),
        ProjectType::Node => RunnerConfig::typescript(), // Node handles JS/TS
        ProjectType::Python => RunnerConfig::python(),
        _ => RunnerConfig::default(),
    }
}

/// Filters files by path pattern if specified.
fn filter_files(files: &[PathBuf], filter: Option<&str>) -> Vec<PathBuf> {
    match filter {
        Some(pattern) => {
            let pattern = pattern.replace('\\', "/");
            files
                .iter()
                .filter(|f| {
                    let s = f.to_string_lossy().replace('\\', "/");
                    s.contains(&pattern)
                })
                .cloned()
                .collect()
        }
        None => files.to_vec(),
    }
}

/// Discovers mutations in all target files.
fn discover_all_mutations(
    files: &[PathBuf],
) -> Vec<mutations::MutationPoint> {
    let mut all_points = Vec::new();

    for file in files {
        match discovery::discover_mutations(file) {
            Ok(points) => all_points.extend(points),
            Err(e) => {
                eprintln!("Warning: Could not scan {}: {e}", file.display());
            }
        }
    }

    all_points
}

/// Prints the header before mutation testing begins.
fn print_header(files: &[PathBuf], config: &RunnerConfig) {
    println!();
    println!("{}", "MUTATION TESTING [EXPERIMENTAL]".bold().cyan());
    println!("{}", "═".repeat(60));
    println!(
        "  Files:    {}",
        files.len().to_string().cyan()
    );
    println!(
        "  Timeout:  {}s",
        config.timeout_secs.to_string().cyan()
    );
    println!(
        "  Command:  {} {}",
        config.test_command.cyan(),
        config.test_args.join(" ")
    );
    println!("{}", "═".repeat(60));
    println!();
}
