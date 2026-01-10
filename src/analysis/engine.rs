// src/analysis/engine.rs
use crate::config::Config;
use crate::types::ScanReport;
use std::path::PathBuf;

/// Orchestrates the analysis of multiple files.
pub struct RuleEngine {
    config: Config,
}

impl RuleEngine {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Entry point for scanning files.
    #[must_use]
    pub fn scan(&self, files: &[PathBuf]) -> ScanReport {
        crate::analysis::logic::run_scan(&self.config, files)
    }
}