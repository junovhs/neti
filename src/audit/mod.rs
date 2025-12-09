// src/audit/mod.rs
//! Consolidation audit system for identifying code cleanup opportunities.
//!
//! This module provides comprehensive analysis to find:
//! - **Duplication**: Structurally similar code units via AST fingerprinting
//! - **Dead Code**: Unreachable code via call graph analysis
//! - **Patterns**: Repeated idioms that could be extracted
//!
//! # Usage
//!
//! ```ignore
//! use slopchop_core::audit::{self, AuditOptions};
//!
//! let options = AuditOptions::default();
//! let report = audit::run(&options)?;
//! println!("{}", audit::format_report(&report));
//! ```
//!
//! # Algorithm Overview
//!
//! 1. **Discovery**: Find all source files using existing discovery logic
//! 2. **Extraction**: Parse files and extract code units (functions, structs, etc.)
//! 3. **Fingerprinting**: Compute structural fingerprints for each unit
//! 4. **Similarity**: Cluster units by fingerprint similarity
//! 5. **Reachability**: Build call graph and find unreachable code
//! 6. **Patterns**: Match predefined patterns across all files
//! 7. **Scoring**: Convert findings to scored opportunities
//! 8. **Reporting**: Format results for display

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
use types::{CodeUnit, CodeUnitKind, DeadCode, SimilarityCluster};

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

    // Load config and discover files
    let mut config = Config::new();
    config.load_local_config();

    let files = discovery::discover(&config)?;

    // Parse all files and extract code units
    let (all_units, all_patterns, file_contents) = analyze_files(&files, options);

    // Find similarity clusters
    let clusters = if options.detect_duplicates {
        similarity::find_clusters(&all_units)
    } else {
        Vec::new()
    };

    // Detect dead code
    let dead_code = if options.detect_dead_code {
        detect_dead_code(&all_units, &file_contents)
    } else {
        Vec::new()
    };

    // Aggregate patterns
    let repeated_patterns = if options.detect_patterns {
        patterns::aggregate(all_patterns)
    } else {
        Vec::new()
    };

    // Convert to opportunities
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

    // Rank and limit
    opportunities = scoring::rank_opportunities(opportunities);
    opportunities.truncate(options.max_opportunities);

    // Calculate stats
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

/// Analyzes all files, extracting code units and pattern matches.
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

/// Analyzes a single file.
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

    // Extract code units
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

    // Detect patterns
    let pattern_matches = if options.detect_patterns {
        patterns::detect_in_file(&content, path, &tree, grammar)
    } else {
        Vec::new()
    };

    Some((units, pattern_matches, path.to_path_buf(), content))
}

/// Extracts lines from content.
fn extract_lines(content: &str, start: usize, end: usize) -> String {
    content
        .lines()
        .skip(start.saturating_sub(1))
        .take(end - start + 1)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parses unit kind from string.
fn parse_unit_kind(kind: &str) -> CodeUnitKind {
    match kind {
        "function" => CodeUnitKind::Function,
        "method" => CodeUnitKind::Method,
        "struct" => CodeUnitKind::Struct,
        "enum" => CodeUnitKind::Enum,
        "trait" => CodeUnitKind::Trait,
        "impl" => CodeUnitKind::Impl,
        "module" => CodeUnitKind::Module,
        _ => CodeUnitKind::Function,
    }
}

/// Detects dead code using call graph analysis.
fn detect_dead_code(units: &[CodeUnit], file_contents: &HashMap<PathBuf, String>) -> Vec<DeadCode> {
    // Build reference list from all files
    let mut all_refs = Vec::new();

    for (path, content) in file_contents {
        let ext = match path.extension().and_then(|s| s.to_str()) {
            Some(e) => e,
            None => continue,
        };

        let lang = match Lang::from_ext(ext) {
            Some(l) => l,
            None => continue,
        };

        let mut parser = Parser::new();
        if parser.set_language(lang.grammar()).is_err() {
            continue;
        }

        let tree = match parser.parse(content, None) {
            Some(t) => t,
            None => continue,
        };

        let refs = dead_code::extract_references(content, path, &tree);
        for (from, to) in refs {
            all_refs.push((path.clone(), from, to));
        }
    }

    // Known entry points
    let entry_points = vec!["main".to_string(), "run".to_string(), "new".to_string()];

    dead_code::detect(units, &all_refs, &entry_points)
}

/// CLI handler for the audit command.
///
/// # Errors
/// Returns error if audit fails.
pub fn handle_audit(
    format: &str,
    no_dead: bool,
    no_dups: bool,
    no_patterns: bool,
    max: usize,
) -> Result<()> {
    use colored::Colorize;

    println!("{}", "🔍 Running consolidation audit...".cyan());

    let options = AuditOptions {
        detect_dead_code: !no_dead,
        detect_duplicates: !no_dups,
        detect_patterns: !no_patterns,
        format: format.to_string(),
        max_opportunities: max,
        ..Default::default()
    };

    let report = run(&options)?;
    let output = format_report(&report, format);

    println!("{output}");

    // Copy AI-friendly version to clipboard for easy paste
    if format == "terminal" && !report.opportunities.is_empty() {
        let ai_version = report::format_ai_prompt(&report);
        if let Err(e) = crate::clipboard::copy_to_clipboard(&ai_version) {
            eprintln!(
                "{}",
                format!("Note: Could not copy to clipboard: {e}").dimmed()
            );
        } else {
            println!("{}", "✓ AI-friendly summary copied to clipboard".green());
        }
    }

    Ok(())
}
