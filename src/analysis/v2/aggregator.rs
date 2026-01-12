// src/analysis/v2/aggregator.rs
//! Aggregation logic for analysis results.
//! Pure data container to decouple data collection from analysis.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::types::Violation;
use super::scope::Scope;
use super::worker::FileAnalysis;

pub struct Aggregator {
    pub violations: HashMap<PathBuf, Vec<Violation>>,
    pub global_scopes: HashMap<String, Scope>,
    pub path_map: HashMap<String, PathBuf>,
}

impl Default for Aggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl Aggregator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            violations: HashMap::new(),
            global_scopes: HashMap::new(),
            path_map: HashMap::new(),
        }
    }

    pub fn ingest(&mut self, path: &Path, analysis: FileAnalysis) {
        if !analysis.violations.is_empty() {
            self.violations
                .entry(path.to_path_buf())
                .or_default()
                .extend(analysis.violations);
        }
        for (name, scope) in analysis.scopes {
            let key = format!("{}::{}", analysis.path_str, name);
            self.global_scopes.insert(key, scope);
            self.path_map.insert(analysis.path_str.clone(), path.to_path_buf());
        }
    }

    pub fn merge(&mut self, others: HashMap<PathBuf, Vec<Violation>>) {
        for (path, vs) in others {
            self.violations.entry(path).or_default().extend(vs);
        }
    }
}