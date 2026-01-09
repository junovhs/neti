// src/analysis/mod.rs
//! Core analysis logic (The "Rule Engine").

pub mod ast;
pub mod checks;
pub mod metrics;
pub mod safety;
pub mod v2;

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

        let tokens = crate::tokens::Tokenizer::count(&source);
        report.token_count = tokens;

        if tokens > self.config.rules.max_file_tokens
            && !Self::is_ignored(path, &self.config.rules.ignore_tokens_on)
        {
            report.violations.push(Violation::simple(
                1,
                format!(
                    "File size is {} tokens (Limit: {})",
                    tokens, self.config.rules.max_file_tokens
                ),
                "LAW OF ATOMICITY",
            ));
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

        if lang == Lang::Rust {
            Self::check_rust_specifics(lang, &ctx, report);
        }
    }

    fn check_naming(lang: Lang, ctx: &checks::CheckContext, report: &mut FileReport) {
        if let Some(q_str) = Self::get_query(lang, QueryKind::Naming) {
            if let Ok(q) = Query::new(lang.grammar(), q_str) {
                checks::check_naming(ctx, &q, &mut report.violations);
            }
        }
    }

    fn check_complexity(lang: Lang, ctx: &checks::CheckContext, report: &mut FileReport) {
        let defs_str = Self::get_query(lang, QueryKind::Defs);
        let comp_str = Self::get_query(lang, QueryKind::Complexity);

        if let (Some(d_str), Some(c_str)) = (defs_str, comp_str) {
            if let (Ok(d), Ok(c)) = (
                Query::new(lang.grammar(), d_str),
                Query::new(lang.grammar(), c_str),
            ) {
                let score = checks::check_metrics(ctx, &d, &c, &mut report.violations);
                report.complexity_score = score;
            }
        }
    }

    fn check_rust_specifics(
        lang: Lang,
        ctx: &checks::CheckContext,
        report: &mut FileReport,
    ) {
        let banned_query_str = r#"
            (call_expression
                function: (field_expression field: (field_identifier) @method)
                (#match? @method "^(unwrap|expect)$"))
        "#;
        if let Ok(q) = Query::new(lang.grammar(), banned_query_str) {
            checks::check_banned(ctx, &q, &mut report.violations);
        }

        let safety_ctx = safety::CheckContext {
            root: ctx.root,
            source: ctx.source,
            filename: ctx.filename,
            config: ctx.config,
        };
        if let Ok(q) = Query::new(lang.grammar(), "") {
            safety::check_safety(&safety_ctx, &q, &mut report.violations);
        }
    }

    fn get_query(lang: Lang, kind: QueryKind) -> Option<&'static str> {
        let q = lang.query(kind);
        if q.is_empty() { None } else { Some(q) }
    }

    fn is_ignored(path: &std::path::Path, patterns: &[String]) -> bool {
        let path_str = path.to_string_lossy();
        patterns.iter().any(|p| path_str.contains(p))
    }

    fn has_ignore_directive(source: &str) -> bool {
        source.lines().take(5).any(|line| line.contains("slopchop:ignore"))
    }
}
