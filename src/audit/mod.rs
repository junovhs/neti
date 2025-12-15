//! Consolidation audit system for identifying code cleanup opportunities.
//!
//! This module provides comprehensive analysis to find:
//! - **Duplication**: Structurally similar code units via AST fingerprinting
//! - **Dead Code**: Unreachable code via call graph analysis
//! - **Patterns**: Repeated idioms that could be extracted

pub mod callsites;
pub mod cfg;
pub mod codegen;
pub mod dead_code;
pub mod diff;
pub mod display;
pub mod enhance;
pub mod fingerprint;
pub mod fp_similarity;
pub mod parameterize;
pub mod patterns;
pub mod report;
pub mod scoring;
pub mod similarity;
pub mod similarity_core;
pub mod types;

// Re-export AuditReport so cli/audit.rs can find it via crate::audit::AuditReport
pub use types::AuditReport;

use crate::audit::report::format_terminal;
use crate::audit::scoring::rank_opportunities;
use crate::config::Config;
use crate::discovery;
use crate::lang::Lang;
use crate::spinner::Spinner;
use anyhow::Result;
use rayon::prelude::*;
use std::path::PathBuf;
use std::time::Instant;

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

/// Container for all analysis results from a single file.
/// This prevents re-parsing the file multiple times.
#[derive(Default)]
struct AnalysisResult {
    units: Vec<types::CodeUnit>,
    references: Vec<(PathBuf, String, String)>,
    patterns: Vec<patterns::PatternMatch>,
}

/// Runs the consolidation audit.
///
/// # Errors
/// Returns error if file discovery or parsing fails.
pub fn run(options: &AuditOptions) -> Result<AuditReport> {
    let start_time = Instant::now();
    let config = Config::default();

    let _spin = Spinner::start("Discovering files...");
    let files = discovery::discover(&config)?;
    let file_count = files.len();

    // Single-pass analysis
    let _spin = Spinner::start("Analyzing code...");
    let results: Vec<AnalysisResult> = files
        .par_iter()
        .flat_map(|path| analyze_file(path, options))
        .collect();

    // Aggregate results
    let mut all_units = Vec::new();
    let mut all_references = Vec::new();
    let mut all_patterns = Vec::new();

    for res in results {
        all_units.extend(res.units);
        all_references.extend(res.references);
        all_patterns.extend(res.patterns);
    }

    let unit_count = all_units.len();
    let mut opportunities = Vec::new();
    let mut stats = types::AuditStats {
        files_analyzed: file_count,
        units_extracted: unit_count,
        ..Default::default()
    };

    // 1. Similarity / Duplication
    if options.detect_duplicates {
        let _spin = Spinner::start("Analyzing similarity...");
        let clusters = similarity::find_clusters(&all_units);
        stats.similarity_clusters = clusters.len();
        stats.total_potential_savings += clusters
            .iter()
            .map(|c| c.potential_savings)
            .sum::<usize>();

        for (i, cluster) in clusters.iter().enumerate() {
            let opp = scoring::score_duplication(cluster, &format!("DUP-{:03}", i + 1));
            opportunities.push(opp);
        }
    }

    // 2. Dead Code
    if options.detect_dead_code {
        let _spin = Spinner::start("Detecting dead code...");
        let dead_code_results =
            dead_code::detect(&all_units, &all_references, &["main".to_string()]);

        stats.dead_code_units = dead_code_results.len();
        for (i, dead) in dead_code_results.iter().enumerate() {
            let opp = scoring::score_dead_code(dead, &format!("DEAD-{:03}", i + 1));
            opportunities.push(opp);
        }
    }

    // 3. Patterns
    if options.detect_patterns {
        let _spin = Spinner::start("Scanning patterns...");
        let repeated = patterns::aggregate(all_patterns);

        stats.pattern_instances = repeated.len();
        for (i, pattern) in repeated.iter().enumerate() {
            let opp = scoring::score_pattern(pattern, &format!("PAT-{:03}", i + 1));
            opportunities.push(opp);
        }
    }

    // Enhance top opportunities with plans
    enhance::enhance_opportunities(&mut opportunities, 5, &config);

    let ranked = rank_opportunities(opportunities);
    let final_opps = ranked
        .into_iter()
        .take(options.max_opportunities)
        .collect();

    stats.duration_ms = start_time.elapsed().as_millis();

    Ok(types::AuditReport {
        opportunities: final_opps,
        stats,
    })
}

fn analyze_file(path: &PathBuf, options: &AuditOptions) -> Option<AnalysisResult> {
    let content = std::fs::read_to_string(path).ok()?;
    let lang = Lang::from_ext(path.extension()?.to_str()?)?;

    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(lang.grammar()).is_err() {
        return None;
    }

    let tree = parser.parse(&content, None)?;
    let mut result = AnalysisResult::default();

    // 1. Extract Units (Always needed for core stats, even if dups disabled)
    let raw_units = fingerprint::extract_units(&content, &tree);
    result.units = raw_units
        .into_iter()
        .map(|(name, kind_str, start, end, fp)| {
            let kind = map_kind(kind_str);
            let tokens = estimate_tokens(&content, start, end);
            types::CodeUnit {
                file: path.clone(),
                name,
                kind,
                start_line: start,
                end_line: end,
                fingerprint: fp,
                tokens,
            }
        })
        .collect();

    // 2. Extract References (for Dead Code)
    if options.detect_dead_code {
        let raw_refs = dead_code::analysis::extract_references(&content, path, &tree);
        result.references = raw_refs
            .into_iter()
            .map(|(caller, callee)| (path.clone(), caller, callee))
            .collect();
    }

    // 3. Detect Patterns
    if options.detect_patterns {
        result.patterns = patterns::detect::detect_in_file(&content, path, &tree, lang.grammar());
    }

    Some(result)
}

fn map_kind(kind_str: &str) -> types::CodeUnitKind {
    use types::CodeUnitKind;
    match kind_str {
        "function" => CodeUnitKind::Function,
        "method" => CodeUnitKind::Method,
        "struct" => CodeUnitKind::Struct,
        "enum" => CodeUnitKind::Enum,
        "trait" => CodeUnitKind::Trait,
        "impl" => CodeUnitKind::Impl,
        _ => CodeUnitKind::Module,
    }
}

fn estimate_tokens(content: &str, start_line: usize, end_line: usize) -> usize {
    content
        .lines()
        .skip(start_line.saturating_sub(1))
        .take(end_line - start_line + 1)
        .flat_map(str::split_whitespace)
        .count()
}

/// Formats the report according to the specified format.
#[must_use]
pub fn format_report(report: &types::AuditReport, format: &str) -> String {
    match format {
        "json" => report::json::format_json(report),
        "ai" => report::ai::format_ai_prompt(report),
        _ => format_terminal(report),
    }
}