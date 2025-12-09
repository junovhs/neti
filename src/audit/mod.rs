// src/audit/mod.rs
//! Consolidation audit system for identifying code cleanup opportunities.
//!
//! This module provides comprehensive analysis to find:
//! - **Duplication**: Structurally similar code units via AST fingerprinting
//! - **Dead Code**: Unreachable code via call graph analysis
//! - **Patterns**: Repeated idioms that could be extracted

pub mod dead_code;
pub mod fingerprint;
pub mod patterns;
pub mod report;
pub mod scoring;
pub mod similarity;
pub mod types;

pub use types::{AuditReport, AuditStats, Opportunity, OpportunityKind};

use crate::config::Config;
use crate::discovery;
use crate::lang::Lang;
use crate::tokens::Tokenizer;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tree_sitter::Parser;
use types::{CodeUnit, CodeUnitKind, DeadCode};

/// Options for the consolidation audit.
#[derive(Debug, Clone)]
pub struct AuditOptions {
    /// Include dead code detection.
    pub detect_dead_code: bool,
    /// Include similarity detection.
    pub detect_duplicates: bool,
    /// Include pattern detection.
    pub detect_patterns: bool,
    /// Minimum lines for a unit to be considered.
    pub min_unit_lines: usize,
    /// Output format: "terminal", "json", or "ai".
    pub format: String,
    /// Maximum number of opportunities to report.
    pub max_opportunities: usize,
}

impl Default for AuditOptions {
    fn default() -> Self {
        Self {
            detect_dead_code: true,
            detect_duplicates: true,
            detect_patterns: true,
            min_unit_lines: 5,
            format: "terminal".to_string(),
            max_opportunities: 50,
        }
    }
}

/// Runs the consolidation audit.
///
/// # Errors
/// Returns error if file discovery or parsing fails.
pub fn run(options: &AuditOptions) -> Result<AuditReport> {
    let start = Instant::now();

    let mut config = Config::new();
    config.load_local_config();

    let files = discovery::discover(&config)?;

    let (all_units, all_patterns, file_contents) = analyze_files(&files, options);

    let clusters = if options.detect_duplicates {
        similarity::find_clusters(&all_units)
    } else {
        Vec::new()
    };

    let dead_code = if options.detect_dead_code {
        detect_dead_code(&all_units, &file_contents)
    } else {
        Vec::new()
    };

    let repeated_patterns = if options.detect_patterns {
        patterns::aggregate(all_patterns)
    } else {
        Vec::new()
    };

    let mut opportunities = Vec::new();

    for cluster in &clusters {
        opportunities.push(scoring::score_duplication(cluster, "audit"));
    }

    for dead in &dead_code {
        opportunities.push(scoring::score_dead_code(dead, "audit"));
    }

    for pattern in &repeated_patterns {
        opportunities.push(scoring::score_pattern(pattern, "audit"));
    }

    opportunities = scoring::rank_opportunities(opportunities);
    opportunities.truncate(options.max_opportunities);

    let total_potential_savings: usize = opportunities.iter().map(|o| o.impact.lines_saved).sum();

    let stats = AuditStats {
        files_analyzed: files.len(),
        units_extracted: all_units.len(),
        similarity_clusters: clusters.len(),
        dead_code_units: dead_code.len(),
        pattern_instances: repeated_patterns.iter().map(|p| p.locations.len()).sum(),
        total_potential_savings,
        duration_ms: start.elapsed().as_millis(),
    };

    Ok(AuditReport {
        opportunities,
        stats,
    })
}

/// Formats the report according to the specified format.
#[must_use]
pub fn format_report(report: &AuditReport, format: &str) -> String {
    match format {
        "json" => report::format_json(report),
        "ai" => report::format_ai_prompt(report),
        _ => report::format_terminal(report),
    }
}

fn analyze_files(
    files: &[PathBuf],
    options: &AuditOptions,
) -> (
    Vec<CodeUnit>,
    Vec<patterns::PatternMatch>,
    HashMap<PathBuf, String>,
) {
    let results: Vec<_> = files
        .par_iter()
        .filter_map(|path| analyze_file(path, options))
        .collect();

    let mut all_units = Vec::new();
    let mut all_patterns = Vec::new();
    let mut file_contents = HashMap::new();

    for (units, pattern_matches, path, content) in results {
        all_units.extend(units);
        all_patterns.extend(pattern_matches);
        file_contents.insert(path, content);
    }

    (all_units, all_patterns, file_contents)
}

fn analyze_file(
    path: &Path,
    options: &AuditOptions,
) -> Option<(Vec<CodeUnit>, Vec<patterns::PatternMatch>, PathBuf, String)> {
    let content = fs::read_to_string(path).ok()?;
    let ext = path.extension().and_then(|s| s.to_str())?;
    let lang = Lang::from_ext(ext)?;
    let grammar = lang.grammar();

    let mut parser = Parser::new();
    if parser.set_language(grammar).is_err() {
        return None;
    }

    let tree = parser.parse(&content, None)?;

    let raw_units = fingerprint::extract_units(&content, &tree);
    let units: Vec<CodeUnit> = raw_units
        .into_iter()
        .filter(|(_, _, start, end, _)| end - start + 1 >= options.min_unit_lines)
        .map(|(name, kind, start_line, end_line, fp)| {
            let unit_content = extract_lines(&content, start_line, end_line);
            let tokens = Tokenizer::count(&unit_content);

            CodeUnit {
                file: path.to_path_buf(),
                name,
                kind: parse_unit_kind(kind),
                start_line,
                end_line,
                fingerprint: fp,
                tokens,
            }
        })
        .collect();

    let pattern_matches = if options.detect_patterns {
        patterns::detect_in_file(&content, path, &tree, grammar)
    } else {
        Vec::new()
    };

    Some((units, pattern_matches, path.to_path_buf(), content))
}

fn extract_lines(content: &str, start: usize, end: usize) -> String {
    content
        .lines()
        .skip(start.saturating_sub(1))
        .take(end - start + 1)
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_unit_kind(kind: &str) -> CodeUnitKind {
    match kind {
        "method" => CodeUnitKind::Method,
        "struct" => CodeUnitKind::Struct,
        "enum" => CodeUnitKind::Enum,
        "trait" => CodeUnitKind::Trait,
        "impl" => CodeUnitKind::Impl,
        "module" => CodeUnitKind::Module,
        // "function" and everything else
        _ => CodeUnitKind::Function,
    }
}

fn detect_dead_code(units: &[CodeUnit], file_contents: &HashMap<PathBuf, String>) -> Vec<DeadCode> {
    let mut all_refs = Vec::new();

    for (path, content) in file_contents {
        let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
            continue;
        };

        let Some(lang) = Lang::from_ext(ext) else {
            continue;
        };

        let mut parser = Parser::new();
        if parser.set_language(lang.grammar()).is_err() {
            continue;
        }

        let Some(tree) = parser.parse(content, None) else {
            continue;
        };

        let refs = dead_code::extract_references(content, path, &tree);
        for (from, to) in refs {
            all_refs.push((path.clone(), from, to));
        }
    }

    let entry_points = vec!["main".to_string(), "run".to_string(), "new".to_string()];

    dead_code::detect(units, &all_refs, &entry_points)
}

