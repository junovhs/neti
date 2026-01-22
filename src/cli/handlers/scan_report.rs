// src/cli/handlers/scan_report.rs
//! Scan report display formatting.

use crate::types::ScanReport;
use crate::analysis::v2::{is_small_codebase, small_codebase_threshold};
use colored::Colorize;
use std::cmp::Reverse;
use std::collections::HashMap;

/// Prints a formatted scan report to stdout.
pub fn print(report: &ScanReport) {
    println!();
    print_header(report);
    print_small_codebase_note(report);
    print_complexity_summary(report);
    print_size_summary(report);
    println!();
}

fn print_header(report: &ScanReport) {
    let status = if report.has_errors() {
        format!("{} violations", report.total_violations).red().bold()
    } else {
        "Clean".green().bold()
    };

    println!(
        "{} {} files │ {} tokens │ {}",
        "SCAN".cyan().bold(),
        report.files.len(),
        report.total_tokens,
        status
    );
}

fn print_small_codebase_note(report: &ScanReport) {
    if is_small_codebase(report.files.len()) {
        println!(
            "{}",
            format!(
                "  ℹ Small codebase (<{} files): structural metrics skipped",
                small_codebase_threshold()
            )
            .dimmed()
        );
    }
}

fn print_complexity_summary(report: &ScanReport) {
    let mut complexity: Vec<_> = report
        .files
        .iter()
        .filter(|f| f.complexity_score > 0)
        .map(|f| (&f.path, f.complexity_score))
        .collect();

    if complexity.is_empty() {
        return;
    }

    complexity.sort_by_key(|(_, c)| Reverse(*c));

    println!("\n{}", "Top Complexity:".dimmed());
    for (path, score) in complexity.iter().take(5) {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        let color = if *score > 15 {
            format!("{score:>3}").red()
        } else if *score > 10 {
            format!("{score:>3}").yellow()
        } else {
            format!("{score:>3}").normal()
        };
        println!("  {} {}", color, name.dimmed());
    }
}

fn print_size_summary(report: &ScanReport) {
    let mut sizes: Vec<_> = report
        .files
        .iter()
        .filter(|f| f.token_count > 1000)
        .map(|f| (&f.path, f.token_count))
        .collect();

    if sizes.is_empty() {
        return;
    }

    sizes.sort_by_key(|(_, t)| Reverse(*t));

    println!("\n{}", "Largest Files:".dimmed());
    for (path, tokens) in sizes.iter().take(5) {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        let color = if *tokens > 2000 {
            format!("{tokens:>5}").red()
        } else if *tokens > 1500 {
            format!("{tokens:>5}").yellow()
        } else {
            format!("{tokens:>5}").normal()
        };
        println!("  {} {}", color, name.dimmed());
    }
}

/// Aggregates violations by law type for summary display.
#[must_use]
pub fn aggregate_by_law(report: &ScanReport) -> HashMap<&'static str, usize> {
    let mut counts: HashMap<&'static str, usize> = HashMap::new();
    for file in &report.files {
        for v in &file.violations {
            *counts.entry(v.law).or_insert(0) += 1;
        }
    }
    counts
}