// src/cli/handlers/scan_report.rs
//! Scan report display formatting.

use crate::types::ScanReport;
use crate::analysis::Engine;
use colored::Colorize;
use std::collections::HashMap;

/// Prints a formatted scan report to stdout (Succinct for Console).
pub fn print(report: &ScanReport) {
    println!();
    print_header(report);
    print_small_codebase_note(report);
    print_violating_files_summary(report, 5); // Max 5 lines in console
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
    if report.files.len() < Engine::small_codebase_threshold() {
        println!(
            "{}",
            format!(
                "  ℹ Small codebase (<{} files): structural metrics skipped",
                Engine::small_codebase_threshold()
            )
            .dimmed()
        );
    }
}

fn print_violating_files_summary(report: &ScanReport, limit: usize) {
    let mut violators: Vec<_> = report.files.iter().filter(|f| !f.is_clean()).collect();
    if violators.is_empty() { return; }

    violators.sort_by_key(|f| std::cmp::Reverse(f.violations.len()));

    println!("\n{}", "Violating files:".dimmed());
    let count = violators.len();
    for f in violators.iter().take(limit) {
        let v_count = f.violations.len();
        let color = if v_count > 5 { format!("{v_count:>3}").red() } else { format!("{v_count:>3}").yellow() };
        println!("  {} {}", color, f.path.display().to_string().dimmed());
    }

    if count > limit {
        println!("  ... and {} more. See {} for full detail.", count - limit, "neti-report.txt".yellow());
    }
}

/// Builds a plain-text summary of the scan report for file logging (Full Detail).
#[must_use]
pub fn build_summary_string(report: &ScanReport) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    let status = if report.has_errors() {
        format!("{} violations", report.total_violations)
    } else {
        "Clean".to_string()
    };

    let _ = writeln!(out, "SCAN SUMMARY: {} files | {} tokens | {}", report.files.len(), report.total_tokens, status);

    let mut violators: Vec<_> = report.files.iter().filter(|f| !f.is_clean()).collect();
    if !violators.is_empty() {
        violators.sort_by_key(|f| std::cmp::Reverse(f.violations.len()));
        let _ = writeln!(out, "\nALL VIOLATING FILES:");
        for f in violators {
            let _ = writeln!(out, "  {:>3} violations | {}", f.violations.len(), f.path.display());
        }
    }

    out
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
