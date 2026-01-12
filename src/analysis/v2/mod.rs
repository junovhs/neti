// src/analysis/v2/mod.rs
pub mod cognitive;
pub mod scope;
pub mod visitor;
pub mod rust;
pub mod metrics;
pub mod patterns;
pub mod inspector;
mod worker;

use crate::config::Config;
use crate::types::Violation;
use std::collections::HashMap;
use std::path::PathBuf;

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
        let mut results: HashMap<PathBuf, Vec<Violation>> = HashMap::new();
        let mut global_scopes = HashMap::new();
        let mut path_map = HashMap::new();

        // Phase 1: Local Analysis (Parallelizable in future)
        for path in files {
            if let Some(analysis) = worker::scan_file(path) {
                // Collect local violations
                if !analysis.violations.is_empty() {
                    results.entry(path.clone()).or_default().extend(analysis.violations);
                }

                // Collect scopes for global analysis
                for (name, scope) in analysis.scopes {
                    let key = format!("{}::{}", analysis.path_str, name);
                    global_scopes.insert(key, scope);
                    path_map.insert(analysis.path_str.clone(), path.clone());
                }
            }
        }

        // Phase 2: Global/Deep Analysis (Metrics)
        self.analyze_scopes(&global_scopes, &path_map, &mut results);
        
        results
    }

    fn analyze_scopes(
        &self,
        scopes: &HashMap<String, scope::Scope>,
        path_map: &HashMap<String, PathBuf>,
        results: &mut HashMap<PathBuf, Vec<Violation>>,
    ) {
        let inspector = inspector::Inspector::new(&self.config.rules);

        for (full_name, scope) in scopes {
            // Map scope back to file path
            let path_str = full_name.split("::").next().unwrap_or("");
            let Some(path) = path_map.get(path_str) else { continue };

            // Run metric checks
            let violations = inspector.inspect(scope);
            
            if !violations.is_empty() {
                results.entry(path.clone()).or_default().extend(violations);
            }
        }
    }
}