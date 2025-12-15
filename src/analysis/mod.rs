// src/analysis/mod.rs
//! Core analysis logic (The "Rule Engine").

pub mod ast;
pub mod checks;
pub mod metrics;

use crate::config::Config;
use crate::lang::{Lang, QueryKind};
use crate::types::{FileReport, Violation};
use rayon::prelude::*;
use std::path::PathBuf;
use tree_sitter::{Parser, Query};

pub struct RuleEngine {
    config: Config,
}

impl RuleEngine {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn scan(&self, files: Vec<PathBuf>) -> crate::types::ScanReport {
        let start = std::time::Instant::now();

        let results: Vec<FileReport> = files
            .par_iter()
            .map(|path| self.analyze_file(path))
            .collect();

        let total_violations: usize = results.iter().map(|r| r.violations.len()).sum();
        let total_tokens: usize = results.iter().map(|r| r.token_count).sum();

        crate::types::ScanReport {
            files: results,
            total_violations,
            total_tokens,
            duration_ms: start.elapsed().as_millis(),
        }
    }

    fn analyze_file(&self, path: &std::path::Path) -> FileReport {
        let mut report = FileReport {
            path: path.to_path_buf(),
            token_count: 0,
            complexity_score: 0,
            violations: Vec::new(),
        };

        let Ok(source) = std::fs::read_to_string(path) else {
            return report;
        };

        if Self::has_ignore_directive(&source) {
            return report;
        }

        report.token_count = crate::tokens::Tokenizer::count(&source);
        if report.token_count > self.config.rules.max_file_tokens
            && !Self::is_ignored(path, &self.config.rules.ignore_tokens_on)
        {
            report.violations.push(Violation {
                row: 1,
                message: format!(
                    "File size is {} tokens (Limit: {})",
                    report.token_count, self.config.rules.max_file_tokens
                ),
                law: "LAW OF ATOMICITY",
            });
        }

        if let Some(lang) = Lang::from_ext(path.extension().and_then(|s| s.to_str()).unwrap_or(""))
        {
            self.run_ast_checks(lang, &source, path.to_str().unwrap_or(""), &mut report);
        }

        report
    }

    fn run_ast_checks(&self, lang: Lang, source: &str, filename: &str, report: &mut FileReport) {
        let mut parser = Parser::new();
        if parser.set_language(lang.grammar()).is_err() {
            return;
        }

        let Some(tree) = parser.parse(source, None) else {
            return;
        };
        let root = tree.root_node();

        let ctx = checks::CheckContext {
            root,
            source,
            filename,
            config: &self.config.rules,
        };

        Self::check_naming(lang, &ctx, report);
        Self::check_complexity(lang, &ctx, report);
        Self::check_banned(lang, &ctx, report);
    }

    fn check_naming(lang: Lang, ctx: &checks::CheckContext, report: &mut FileReport) {
        let naming_str = lang.query(QueryKind::Naming);
        if !naming_str.is_empty() {
            if let Ok(q) = Query::new(lang.grammar(), naming_str) {
                checks::check_naming(ctx, &q, &mut report.violations);
            }
        }
    }

    fn check_complexity(lang: Lang, ctx: &checks::CheckContext, report: &mut FileReport) {
        let defs_str = lang.query(QueryKind::Defs);
        let complexity_str = lang.query(QueryKind::Complexity);

        if !defs_str.is_empty() && !complexity_str.is_empty() {
            let defs_q = Query::new(lang.grammar(), defs_str);
            let comp_q = Query::new(lang.grammar(), complexity_str);

            if let (Ok(d), Ok(c)) = (defs_q, comp_q) {
                checks::check_metrics(ctx, &d, &c, &mut report.violations);
            }
        }
    }

    fn check_banned(lang: Lang, ctx: &checks::CheckContext, report: &mut FileReport) {
        if lang == Lang::Rust {
            // Updated to support method_call_expression for modern tree-sitter-rust
            let banned_query_str = r"
                (call_expression
                    function: (field_expression field: (field_identifier) @method)
                    (#match? @method ^(unwrap|expect)$))
                (method_call_expression
                    name: (identifier) @method
                    (#match? @method ^(unwrap|expect)$))
                (method_invocation
                    name: (identifier) @method
                    (#match? @method ^(unwrap|expect)$))
            ";
            if let Ok(q) = Query::new(lang.grammar(), banned_query_str) {
                checks::check_banned(ctx, &q, &mut report.violations);
            }

            if let Ok(q) = Query::new(lang.grammar(), "") {
                checks::check_safety(ctx, &q, &mut report.violations);
            }
        }
    }

    fn is_ignored(path: &std::path::Path, patterns: &[String]) -> bool {
        let path_str = path.to_string_lossy();
        patterns.iter().any(|p| path_str.contains(p))
    }

    fn has_ignore_directive(source: &str) -> bool {
        source
            .lines()
            .take(5)
            .any(|line| line.contains("slopchop:ignore"))
    }
}