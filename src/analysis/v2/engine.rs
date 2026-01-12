// src/analysis/v2/engine.rs
//! Main execution logic for Scan V2.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::Config;
use crate::types::Violation;

use super::aggregator::Aggregator;
use super::deep::DeepAnalyzer;
use super::worker;

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
        let mut aggregator = Aggregator::new();

        // Phase 1: Local Analysis (Parallelizable)
        for path in files {
            if let Some(analysis) = worker::scan_file(path) {
                aggregator.ingest(path, analysis);
            }
        }

        // Phase 2: Global/Deep Analysis (Metrics)
        let deep_analyzer = DeepAnalyzer::new(&self.config.rules);
        let deep_violations = deep_analyzer.compute_violations(&aggregator);
        aggregator.merge(deep_violations);

        aggregator.violations
    }
}