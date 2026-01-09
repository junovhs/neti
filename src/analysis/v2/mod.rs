// src/analysis/v2/mod.rs
pub mod cognitive;
pub mod scope;
pub mod visitor;

use crate::config::Config;
use crate::lang::Lang;
use std::collections::HashMap;
use std::path::PathBuf;
use tree_sitter::Parser;

/// Aggregated metrics for a Scan v2 run.
#[derive(Debug, Default)]
pub struct MetricsV2 {
    pub lcom4_violations: usize,
    pub high_cbo_count: usize,
    pub high_sfout_count: usize,
    pub max_cognitive: usize,
}

#[allow(dead_code)]
pub struct ScanEngineV2 {
    config: Config,
}

impl ScanEngineV2 {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Runs the Scan v2 engine over the provided files.
    #[must_use]
    pub fn run(&self, files: &[PathBuf]) -> MetricsV2 {
        let mut global_scopes = HashMap::new();

        for path in files {
            Self::process_file(path, &mut global_scopes);
        }

        Self::analyze_metrics(&global_scopes)
    }

    fn process_file(path: &PathBuf, global_scopes: &mut HashMap<String, scope::Scope>) {
        let Ok(source) = std::fs::read_to_string(path) else { return; };
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let Some(lang) = Lang::from_ext(ext) else { return; };

        let mut parser = Parser::new();
        if parser.set_language(lang.grammar()).is_err() { return; }
        
        if let Some(tree) = parser.parse(&source, None) {
            let visitor = visitor::AstVisitor::new(&source, lang);
            let file_scopes = visitor.extract_scopes(tree.root_node());
            global_scopes.extend(file_scopes);
        }
    }

    fn analyze_metrics(scopes: &HashMap<String, scope::Scope>) -> MetricsV2 {
        let mut metrics = MetricsV2::default();
        for scope in scopes.values() {
            Self::update_metrics_from_scope(scope, &mut metrics);
        }
        metrics
    }

    fn update_metrics_from_scope(scope: &scope::Scope, metrics: &mut MetricsV2) {
        Self::update_coupling_metrics(scope, metrics);
        
        if scope.calculate_lcom4() > 1 {
            metrics.lcom4_violations += 1;
        }
        
        let max_cog = scope.methods.values().map(|m| m.cognitive_complexity).max().unwrap_or(0);
        if max_cog > metrics.max_cognitive {
            metrics.max_cognitive = max_cog;
        }
    }

    fn update_coupling_metrics(scope: &scope::Scope, metrics: &mut MetricsV2) {
        if scope.calculate_cbo() > 9 {
            metrics.high_cbo_count += 1;
        }
        if scope.calculate_max_sfout() > 7 {
            metrics.high_sfout_count += 1;
        }
    }
}