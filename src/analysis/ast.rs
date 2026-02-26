use super::checks::{self, CheckContext};
use super::cognitive::CognitiveAnalyzer;
use crate::config::RuleConfig;
use crate::lang::{Lang, QueryKind};
use crate::types::{Violation, ViolationDetails};
use anyhow::{anyhow, Result};
use tree_sitter::{Language, Parser, Query, QueryCursor};

pub struct Analyzer;

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AnalysisResult {
    pub violations: Vec<Violation>,
    pub max_complexity: usize,
}

impl Analyzer {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn analyze(
        &self,
        lang: Lang,
        filename: &str,
        content: &str,
        config: &RuleConfig,
    ) -> AnalysisResult {
        Self::run_analysis(lang, filename, content, config)
    }

    fn run_analysis(
        lang: Lang,
        filename: &str,
        content: &str,
        config: &RuleConfig,
    ) -> AnalysisResult {
        let grammar = lang.grammar();
        let mut parser = Parser::new();
        if parser.set_language(&grammar).is_err() {
            return AnalysisResult {
                violations: vec![],
                max_complexity: 0,
            };
        }

        let Some(tree) = parser.parse(content, None) else {
            return AnalysisResult {
                violations: vec![],
                max_complexity: 0,
            };
        };

        let root = tree.root_node();
        let mut violations = Vec::new();
        let ctx = CheckContext {
            root,
            source: content,
            filename,
            config,
        };

        if let Ok(q) = compile_query(&grammar, lang.query(QueryKind::Naming)) {
            checks::check_naming(&ctx, &q, &mut violations);
        }

        let max_complexity = compute_max_complexity(&grammar, lang, &ctx, &mut violations);

        checks::check_syntax(&ctx, &mut violations);

        if lang == Lang::Rust {
            Self::check_rust_specifics(&grammar, &ctx, &mut violations);
        }

        AnalysisResult {
            violations,
            max_complexity,
        }
    }

    fn check_rust_specifics(grammar: &Language, ctx: &CheckContext, out: &mut Vec<Violation>) {
        let banned_query_str = r"
            (call_expression
                function: (field_expression field: (field_identifier) @method)
                (#match? @method ^(unwrap|expect)$))
        ";
        if let Ok(q) = compile_query(grammar, banned_query_str) {
            checks::check_banned(ctx, &q, out);
        }

        if let Ok(q) = compile_query(grammar, "") {
            super::safety::check_safety(ctx, &q, out);
        }
    }
}

fn score_capture(
    cap: &tree_sitter::QueryCapture,
    ctx: &CheckContext,
    violations: &mut Vec<Violation>,
) -> usize {
    let node = cap.node;
    let kind = node.kind();
    if !matches!(
        kind,
        "function_item" | "function_definition" | "method_definition" | "function_declaration"
    ) {
        return 0;
    }

    let score = CognitiveAnalyzer::calculate(node, ctx.source);

    if score > ctx.config.max_cognitive_complexity {
        let name = node
            .child_by_field_name("name")
            .and_then(|n| n.utf8_text(ctx.source.as_bytes()).ok())
            .unwrap_or("<anonymous>");

        violations.push(Violation::with_details(
            node.start_position().row + 1,
            format!(
                "Function '{name}' has cognitive complexity {score} (Max: {})",
                ctx.config.max_cognitive_complexity
            ),
            "LAW OF COMPLEXITY",
            ViolationDetails {
                function_name: Some(name.to_string()),
                analysis: vec![format!("Cognitive score: {score}")],
                suggestion: Some("Break logic into smaller, linear functions.".into()),
            },
        ));
    }
    score
}

fn compute_max_complexity(
    grammar: &Language,
    lang: Lang,
    ctx: &CheckContext,
    violations: &mut Vec<Violation>,
) -> usize {
    let Ok(q_defs) = compile_query(grammar, lang.query(QueryKind::Defs)) else {
        return 0;
    };
    let mut cursor = QueryCursor::new();
    let captures: Vec<_> = cursor
        .matches(&q_defs, ctx.root, ctx.source.as_bytes())
        .flat_map(|m| m.captures.iter().copied())
        .collect();

    let mut max_complexity = 0;
    for cap in &captures {
        let score = score_capture(cap, ctx, violations);
        if score > max_complexity {
            max_complexity = score;
        }
    }
    max_complexity
}

fn compile_query(lang: &Language, pattern: &str) -> Result<Query> {
    Query::new(lang, pattern).map_err(|e| anyhow!("Invalid tree-sitter query: {e}"))
}
