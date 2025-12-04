// src/analysis/mod.rs
pub mod ast;
pub mod checks;
pub mod metrics;

use crate::config::Config;
use crate::tokens::Tokenizer;
use crate::types::{FileReport, ScanReport, Violation};
use ast::Analyzer;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::Instant;

static ANALYZER: LazyLock<Analyzer> = LazyLock::new(Analyzer::new);

pub struct RuleEngine {
    config: Config,
}

impl RuleEngine {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Scans a list of files and returns a structured report.
    #[must_use]
    pub fn scan(&self, files: Vec<PathBuf>) -> ScanReport {
        let start = Instant::now();

        let results: Vec<FileReport> = files
            .into_par_iter()
            .filter_map(|path| self.analyze_file(&path))
            .collect();

        let total_tokens = results.iter().map(|f| f.token_count).sum();
        let total_violations = results.iter().map(|f| f.violations.len()).sum();

        ScanReport {
            files: results,
            total_tokens,
            total_violations,
            duration_ms: start.elapsed().as_millis(),
        }
    }

    fn analyze_file(&self, path: &Path) -> Option<FileReport> {
        let content = fs::read_to_string(path).ok()?;

        // Support C-style, Hash-style, and HTML-style (Markdown) ignores
        if content.contains("// slopchop:ignore")
            || content.contains("# slopchop:ignore")
            || content.contains("<!-- slopchop:ignore -->")
        {
            return None;
        }

        let filename = path.to_string_lossy();
        let token_count = Tokenizer::count(&content);
        let mut violations = Vec::new();

        // 1. Law of Atomicity (checked unless exempted)
        if !self.is_exempt_from_tokens(&filename) && token_count > self.config.rules.max_file_tokens
        {
            violations.push(Violation {
                row: 0,
                message: format!(
                    "File size is {token_count} tokens (Limit: {})",
                    self.config.rules.max_file_tokens
                ),
                law: "LAW OF ATOMICITY",
            });
        }

        // 2. AST Analysis (complexity, nesting, arity, banned calls)
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            let mut ast_violations = ANALYZER.analyze(ext, &filename, &content, &self.config.rules);
            violations.append(&mut ast_violations);
        }

        Some(FileReport {
            path: path.to_path_buf(),
            token_count,
            complexity_score: 0,
            violations,
        })
    }

    fn is_exempt_from_tokens(&self, filename: &str) -> bool {
        self.config
            .rules
            .ignore_tokens_on
            .iter()
            .any(|pattern| filename.contains(pattern))
    }
}
