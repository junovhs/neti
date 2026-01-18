// src/analysis/engine.rs
use crate::config::Config;
use crate::types::ScanReport;
use std::path::{Path, PathBuf};

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
        // Default to no progress callback for backward compatibility if needed,
        // but logic.rs now takes optional.
        crate::analysis::logic::run_scan(&self.config, files, None::<&fn(&Path)>)
    }

    /// Entry point for scanning files with progress callback.
    pub fn scan_with_progress<F>(&self, files: &[PathBuf], on_progress: &F) -> ScanReport
    where
        F: Fn(&Path) + Sync,
    {
        crate::analysis::logic::run_scan(&self.config, files, Some(on_progress))
    }
}