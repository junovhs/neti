//! Worker module for file parsing and analysis.

use std::collections::HashMap;
use std::path::Path;

use tree_sitter::Parser;

use crate::config::Config;
use crate::lang::Lang;
use crate::tokens::Tokenizer;
use crate::types::{FileReport, Violation};

use super::aggregator::FileAnalysis;
use super::ast;
use super::patterns;
use super::visitor::AstVisitor;

#[must_use]
pub fn scan_file(path: &Path, config: &Config) -> FileReport {
    let mut report = FileReport {
        path: path.to_path_buf(),
        token_count: 0,
        complexity_score: 0,
        violations: Vec::new(),
        analysis: None,
    };

    let Ok(source) = std::fs::read_to_string(path) else {
        return report;
    };

    report.token_count = Tokenizer::count(&source);

    let effective_config = determine_effective_config(&source, config);

    if report.token_count > effective_config.rules.max_file_tokens
        && !is_ignored(path, &effective_config.rules.ignore_tokens_on)
    {
        report.violations.push(Violation::simple(
            1,
            format!(
                "File size is {} tokens (Limit: {})",
                report.token_count, effective_config.rules.max_file_tokens
            ),
            "LAW OF ATOMICITY",
        ));
    }

    let Some(lang) = Lang::from_ext(path.extension().and_then(|s| s.to_str()).unwrap_or("")) else {
        return report;
    };

    let mut parser = Parser::new();
    if parser.set_language(lang.grammar()).is_err() {
        return report;
    }

    let Some(tree) = parser.parse(&source, None) else {
        return report;
    };

    let root = tree.root_node();

    report
        .violations
        .extend(patterns::detect_all(path, &source));

    let ast_result = ast::Analyzer::new().analyze(
        lang,
        path.to_str().unwrap_or(""),
        &source,
        &effective_config.rules,
    );
    report.violations.extend(ast_result.violations);
    report.complexity_score = ast_result.max_complexity;

    let scopes = if lang == Lang::Rust {
        let visitor = AstVisitor::new(&source, lang);
        visitor.extract_scopes(root)
    } else {
        HashMap::new()
    };

    report.analysis = Some(FileAnalysis {
        path_str: path.to_string_lossy().to_string(),
        scopes,
        violations: Vec::new(),
    });

    report
}

fn determine_effective_config(source: &str, base_config: &Config) -> Config {
    let score = calculate_systems_score(source);
    if score >= 3 {
        let mut sys_cfg = base_config.clone();
        sys_cfg.rules.max_file_tokens = 10_000;
        sys_cfg.rules.max_cognitive_complexity = 50;
        sys_cfg.rules.max_lcom4 = 100;
        sys_cfg.rules.max_cbo = 100;
        sys_cfg
    } else {
        base_config.clone()
    }
}

fn calculate_systems_score(source: &str) -> usize {
    let mut score = 0;
    if source.contains("#![no_std]") {
        score += 5;
    }
    if source.contains("unsafe {") || source.contains("unsafe fn") {
        score += 1;
    }
    if source.contains("transmute") {
        score += 2;
    }
    if source.contains("repr(C)") || source.contains("repr(packed)") {
        score += 2;
    }
    if source.contains("Atomic") {
        score += 1;
    }
    if source.contains("*const") || source.contains("*mut") {
        score += 1;
    }
    if source.contains("Pin<Box") {
        score += 1;
    }
    score
}

#[must_use]
pub fn is_ignored(path: &Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    patterns.iter().any(|p| path_str.contains(p))
}
