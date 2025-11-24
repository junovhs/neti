use crate::analysis::Analyzer;
use crate::config::Config;
use crate::tokens::Tokenizer;
use crate::types::{FileReport, ScanReport, Violation};
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
    ///
    /// # Errors
    ///
    /// Returns error if Rayon thread pool fails (unlikely).
    #[must_use]
    pub fn scan(&self, files: Vec<PathBuf>) -> ScanReport {
        let start = Instant::now();

        // Parallel scan
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

        if content.contains("// warden:ignore") || content.contains("# warden:ignore") {
            return None;
        }

        let filename = path.to_string_lossy();
        let mut violations = Vec::new();
        let token_count = Tokenizer::count(&content);

        // 1. Law of Atomicity
        if token_count > self.config.rules.max_file_tokens {
            violations.push(Violation {
                row: 0,
                message: format!(
                    "File size is {token_count} tokens (Limit: {})",
                    self.config.rules.max_file_tokens
                ),
                law: "LAW OF ATOMICITY",
            });
        }

        // 2. AST Analysis
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            let mut analysis_violations =
                ANALYZER.analyze(ext, &filename, &content, &self.config.rules);
            violations.append(&mut analysis_violations);
        }

        Some(FileReport {
            path: path.to_path_buf(),
            token_count,
            complexity_score: 0, // Future: aggregate function complexity here
            violations,
        })
    }
}
