// src/analysis/v2/engine.rs
//! Main execution logic for Scan V2.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::Config;
use crate::types::Violation;

use super::aggregator::Aggregator;
use super::deep::DeepAnalyzer;
use super::worker;

/// Files below this threshold skip structural metrics (LCOM4, CBO, AHF, SFOUT).
/// Rationale: For small projects, modularity metrics are noise, not signal.
/// See: case-study-gittrek.md
pub const SMALL_CODEBASE_THRESHOLD: usize = 10;

/// Returns true if the file count is below the small codebase threshold.
#[must_use]
pub fn is_small_codebase(file_count: usize) -> bool {
    file_count < SMALL_CODEBASE_THRESHOLD
}

/// Returns the small codebase threshold for external display.
#[must_use]
pub const fn small_codebase_threshold() -> usize {
    SMALL_CODEBASE_THRESHOLD
}

pub struct ScanEngineV2 {
    config: Config,
}

impl ScanEngineV2 {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Runs the Scan v2 engine and returns violations mapped by file path.
    #[must_use]
    pub fn run(&self, files: &[PathBuf]) -> HashMap<PathBuf, Vec<Violation>> {
        run_analysis(files, &self.config)
    }
}

/// Core analysis logic extracted to reduce struct fan-out.
fn run_analysis(files: &[PathBuf], config: &Config) -> HashMap<PathBuf, Vec<Violation>> {
    // Small codebase detection: skip structural metrics entirely.
    if files.len() < SMALL_CODEBASE_THRESHOLD {
        return HashMap::new();
    }

    let aggregator = collect_local_analysis(files);
    compute_deep_violations(aggregator, config)
}

/// Phase 1: Local analysis (parallelizable).
fn collect_local_analysis(files: &[PathBuf]) -> Aggregator {
    let mut aggregator = Aggregator::new();
    for path in files {
        if let Some(analysis) = worker::scan_file(path) {
            aggregator.ingest(path, analysis);
        }
    }
    aggregator
}

/// Phase 2: Global/Deep analysis (metrics).
fn compute_deep_violations(
    mut aggregator: Aggregator,
    config: &Config,
) -> HashMap<PathBuf, Vec<Violation>> {
    let deep_analyzer = DeepAnalyzer::new(&config.rules);
    let deep_violations = deep_analyzer.compute_violations(&aggregator);
    aggregator.merge(deep_violations);
    aggregator.violations
}