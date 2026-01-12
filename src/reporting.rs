// src/reporting.rs
//! Console output formatting for scan results.

use crate::types::{FileReport, ScanReport, Violation};
use anyhow::Result;
use colored::Colorize;
use std::fmt::Write;
use std::fs;
use std::path::Path;
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
        print_violation(&file.path, v);
    }
}

fn print_violation(path: &Path, v: &Violation) {
    let path_str = path.display().to_string();
    println!(
        "{} {}",
        "error:".red().bold(),
        v.message
    );
    println!(
        "  {} {}:{}",
        "-->".blue(),
        path_str,
        v.row
    );

    // Render code snippet if available
    print_snippet(path, v.row);

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

fn print_snippet(path: &Path, row: usize) {
    // Basic caching could go here, but OS file cache is usually sufficient for CLI
    let Ok(content) = fs::read_to_string(path) else { return };
    let lines: Vec<&str> = content.lines().collect();
    
    // Convert 1-based row to 0-based index
    let idx = row.saturating_sub(1);
    
    // Show 1 line of context above, the error line, and 1 below if possible
    let start = idx.saturating_sub(1);
    let end = (idx + 1).min(lines.len() - 1);

    println!("   {}", "|".blue());
    
    for i in start..=end {
        if let Some(line) = lines.get(i) {
            let line_num = i + 1;
            let gutter = format!("{line_num:3} |");
            
            if i == idx {
                // Focus line
                println!("   {} {}", gutter.blue(), line);
                
                // Draw underline
                // Simple heuristic: underline everything that isn't leading whitespace
                let trimmed = line.trim_start();
                let padding = line.len() - trimmed.len();
                let underline_len = trimmed.len().max(1);
                let spaces = " ".repeat(padding);
                let carets = "^".repeat(underline_len);
                
                println!("   {} {}{}", "|".blue(), spaces, carets.red().bold());
            } else {
                // Context line
                println!("   {} {}", gutter.blue().dimmed(), line.dimmed());
            }
        }
    }
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