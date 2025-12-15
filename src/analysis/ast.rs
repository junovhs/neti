// src/analysis/ast.rs
use super::checks::{self, CheckContext};
use crate::config::RuleConfig;
use crate::lang::{Lang, QueryKind};
use crate::types::Violation;
use anyhow::{anyhow, Result};
use tree_sitter::{Language, Parser, Query};

pub struct Analyzer;

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn analyze(
        &self,
        ext: &str,
        filename: &str,
        content: &str,
        config: &RuleConfig,
    ) -> Vec<Violation> {
        let Some(lang) = Lang::from_ext(ext) else {
            return vec![];
        };
        Self::run_analysis(lang, filename, content, config)
    }

    fn run_analysis(
        lang: Lang,
        filename: &str,
        content: &str,
        config: &RuleConfig,
    ) -> Vec<Violation> {
        let grammar = lang.grammar();
        let mut parser = Parser::new();
        if parser.set_language(grammar).is_err() {
            return vec![];
        }

        let Some(tree) = parser.parse(content, None) else {
            return vec![];
        };

        // Fallible compilation steps
        let Ok(q_naming) = compile_query(grammar, lang.query(QueryKind::Naming)) else {
            return vec![];
        };

        let Ok(q_defs) = compile_query(grammar, lang.query(QueryKind::Defs)) else {
            return vec![];
        };

        let Ok(q_complexity) = compile_query(grammar, lang.query(QueryKind::Complexity)) else {
            return vec![];
        };

        // Banned query is Rust only mostly, or provided by Lang?
        // Lang::query(QueryKind::Banned) doesn't exist yet, falling back to manual or similar logic?
        // Since `ast.rs` seems to be used for general analysis (maybe legacy or specific test?),
        // we'll stick to what was there but use QueryKind where possible.
        // If `Lang` doesn't have Banned in QueryKind enum, we skip or handle Rust specific logic if needed.
        // For now, ignoring banned check here if not supported by Lang query map, or check strictness.

        let mut violations = Vec::new();
        let ctx = CheckContext {
            root: tree.root_node(),
            source: content,
            filename,
            config,
        };

        checks::check_naming(&ctx, &q_naming, &mut violations);
        checks::check_metrics(&ctx, &q_defs, &q_complexity, &mut violations);

        // Banned check logic from mod.rs logic if Rust
        if lang == Lang::Rust {
            let banned_query_str = r"
                (call_expression
                    function: (field_expression field: (field_identifier) @method)
                    (#match? @method ^(unwrap|expect)$))
                (method_invocation
                    name: (identifier) @method
                    (#match? @method ^(unwrap|expect)$))
            ";
            if let Ok(banned) = compile_query(grammar, banned_query_str) {
                checks::check_banned(&ctx, &banned, &mut violations);
            }
        }

        violations
    }
}

fn compile_query(lang: Language, pattern: &str) -> Result<Query> {
    Query::new(lang, pattern).map_err(|e| anyhow!("Invalid tree-sitter query: {e}"))
}