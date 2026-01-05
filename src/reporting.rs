// src/reporting.rs
//! Console output formatting for scan results.

use crate::types::{FileReport, ScanReport, Violation};
use anyhow::Result;
use colored::Colorize;
use std::fmt::Write;
use std::time::Duration;

/// Prints a formatted scan report to stdout.
///
/// # Errors
/// Returns error if formatting fails.
pub fn print_report(report: &ScanReport) -> Result<()> {
    if report.has_errors() {
        print_violations(report);
    }
    print_summary(report);
    Ok(())
}

fn print_violations(report: &ScanReport) {
    for file in report.files.iter().filter(|f| !f.is_clean()) {
        print_file_violations(file);
    }
}

fn print_file_violations(file: &FileReport) {
    for v in &file.violations {
        print_violation(&file.path.display().to_string(), v);
    }
}

fn print_violation(path: &str, v: &Violation) {
    println!(
        "{} {}",
        "error:".red().bold(),
        v.message
    );
    println!(
        "  {} {}:{}",
        "-->".blue(),
        path,
        v.row
    );
    println!("   {}", "|".blue());
    println!(
        "   {} {}: Action required",
        "=".blue(),
        v.law.yellow()
    );

    if let Some(ref details) = v.details {
        print_violation_details(details);
    }

    println!();
}

fn print_violation_details(details: &crate::types::ViolationDetails) {
    if !details.analysis.is_empty() {
        println!("   {}", "|".blue());
        println!("   {} {}", "=".blue(), "ANALYSIS:".cyan());
        for line in &details.analysis {
            println!("   {}   {}", "|".blue(), line.dimmed());
        }
    }

    if let Some(ref suggestion) = details.suggestion {
        println!("   {}", "|".blue());
        println!(
            "   {} {} {}",
            "=".blue(),
            "SUGGESTION:".green(),
            suggestion
        );
    }
}

fn print_summary(report: &ScanReport) {
    #[allow(clippy::cast_possible_truncation)]
    let duration = Duration::from_millis(report.duration_ms as u64);

    if report.has_errors() {
        println!(
            "{} SlopChop found {} {} in {:?}.",
            "X".red().bold(),
            report.total_violations,
            pluralize("violation", report.total_violations),
            duration
        );
    } else {
        println!(
            "{} No violations found in {:?}.",
            "OK".green().bold(),
            duration
        );
    }
}

fn pluralize(word: &str, count: usize) -> String {
    if count == 1 {
        word.to_string()
    } else {
        format!("{word}s")
    }
}

/// Formats a report as a string (for embedding in context files).
///
/// # Errors
/// Returns error if formatting fails.
pub fn format_report_string(report: &ScanReport) -> Result<String> {
    let mut out = String::new();

    for file in report.files.iter().filter(|f| !f.is_clean()) {
        for v in &file.violations {
            writeln!(
                out,
                "FILE: {} | LAW: {} | LINE: {} | {}",
                file.path.display(),
                v.law,
                v.row,
                v.message
            )?;
        }
    }

    Ok(out)
}

/// Prints a serializable object as JSON to stdout.
///
/// # Errors
/// Returns error if serialization fails.
pub fn print_json<T: serde::Serialize>(data: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    println!("{json}");
    Ok(())
}