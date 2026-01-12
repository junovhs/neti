// src/analysis/v2/deep.rs
//! Deep analysis runner. Separated to reduce coupling in Engine.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::RuleConfig;
use crate::types::Violation;

use super::aggregator::Aggregator;
use super::inspector::Inspector;

pub struct DeepAnalyzer<'a> {
    config: &'a RuleConfig,
}

impl<'a> DeepAnalyzer<'a> {
    #[must_use]
    pub fn new(config: &'a RuleConfig) -> Self {
        Self { config }
    }

    #[must_use]
    pub fn compute_violations(&self, agg: &Aggregator) -> HashMap<PathBuf, Vec<Violation>> {
        let mut results: HashMap<PathBuf, Vec<Violation>> = HashMap::new();
        let inspector = Inspector::new(self.config);

        for (full_name, scope) in &agg.global_scopes {
            let path_str = full_name.split("::").next().unwrap_or("");
            if let Some(path) = agg.path_map.get(path_str) {
                let vs = inspector.inspect(scope);
                if !vs.is_empty() {
                    results.entry(path.clone()).or_default().extend(vs);
                }
            }
        }
        results
    }
}