//! Main execution logic for the `Neti` analysis engine.
//! Unified entry point for all scanning operations.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::config::Config;
use crate::types::{FileReport, ScanReport, Violation};

use super::aggregator::Aggregator;
use super::deep::DeepAnalyzer;
use super::worker;

/// Source files below this threshold skip structural metrics (LCOM4, CBO, AHF, SFOUT).
/// Rationale: For small projects, modularity metrics are noise, not signal.
pub const SMALL_CODEBASE_THRESHOLD: usize = 10;

/// The main analysis engine.
/// Orchestrates file scanning, pattern detection, and structural analysis.
pub struct Engine {
    config: Config,
}

impl Engine {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Returns the small codebase threshold for external display.
    #[must_use]
    pub const fn small_codebase_threshold() -> usize {
        SMALL_CODEBASE_THRESHOLD
    }

    /// Entry point for scanning files with progress callbacks.
    pub fn scan_with_progress<F, S>(
        &self,
        files: &[PathBuf],
        on_progress: &F,
        on_status: &S,
    ) -> ScanReport
    where
        F: Fn(&Path) + Sync,
        S: Fn(&str) + Sync,
    {
        let start = std::time::Instant::now();

        // Phase 1: Local Analysis (Parallel)
        // Token counts, patterns, basic checks, scope extraction
        let mut results: Vec<FileReport> = files
            .par_iter()
            .inspect(|path| {
                on_progress(path);
            })
            .map(|path| worker::scan_file(path, &self.config))
            .collect();

        // Phase 2: Deep Analysis (Sequential/Aggregated)
        // Structural metrics (LCOM4, CBO) for sufficiently large codebases
        on_status("Running Deep Analysis (LCOM4/CBO)...");

        if should_run_deep_analysis(&results) {
            let deep_violations = self.run_deep_analysis(&results);
            merge_violations(&mut results, &deep_violations);
        }

        ScanReport {
            total_violations: results.iter().map(|r| r.violations.len()).sum(),
            total_tokens: results.iter().map(|r| r.token_count).sum(),
            files: results,
            duration_ms: start.elapsed().as_millis(),
        }
    }

    /// Entry point for scanning files without progress callbacks.
    #[must_use]
    pub fn scan(&self, files: &[PathBuf]) -> ScanReport {
        let start = std::time::Instant::now();

        let mut results: Vec<FileReport> = files
            .par_iter()
            .map(|path| worker::scan_file(path, &self.config))
            .collect();

        if should_run_deep_analysis(&results) {
            let deep_violations = self.run_deep_analysis(&results);
            merge_violations(&mut results, &deep_violations);
        }

        ScanReport {
            total_violations: results.iter().map(|r| r.violations.len()).sum(),
            total_tokens: results.iter().map(|r| r.token_count).sum(),
            files: results,
            duration_ms: start.elapsed().as_millis(),
        }
    }

    fn run_deep_analysis(&self, results: &[FileReport]) -> HashMap<PathBuf, Vec<Violation>> {
        // Aggregate scopes from all files
        let mut aggregator = Aggregator::new();
        for report in results {
            if let Some(analysis) = &report.analysis {
                aggregator.ingest(&report.path, analysis);
            }
        }

        // Run deep inspector
        let deep_analyzer = DeepAnalyzer::new(&self.config.rules);
        deep_analyzer.compute_violations(&aggregator)
    }
}

fn should_run_deep_analysis(results: &[FileReport]) -> bool {
    // Small codebase detection: count only src/ files, skip structural metrics.
    let source_count = results.iter().filter(|r| is_source_file(&r.path)).count();
    source_count >= SMALL_CODEBASE_THRESHOLD
}

fn merge_violations(results: &mut [FileReport], deep: &HashMap<PathBuf, Vec<Violation>>) {
    for r in results {
        if let Some(v) = deep.get(&r.path) {
            r.violations.extend(v.clone());
        }
    }
}

/// Returns true if path is a source file (not test/bench/example).
fn is_source_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    if !path_str.contains("src/") && !path_str.starts_with("src/") {
        return false;
    }

    if path_str.contains("/tests/") || path_str.contains("_test.") || path_str.contains("tests.rs")
    {
        return false;
    }

    true
}
