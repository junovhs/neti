// src/cli/handlers/scan_report.rs
//! Scan report display formatting.

use crate::types::ScanReport;
use colored::Colorize;
use std::cmp::Reverse;
use std::collections::HashMap;

/// Prints the scan analysis report.
pub fn print(report: &ScanReport) {
    println!();
    println!("{}", "─".repeat(60).dimmed());
    println!("{}", " SCAN REPORT".bold());
    println!("{}", "─".repeat(60).dimmed());
    println!();

    print_stats(report);

    if report.has_errors() {
        println!();
        print_violations(report);
        print_offenders(report);
    }

    println!();
    println!("{}", "─".repeat(60).dimmed());
}

fn print_stats(report: &ScanReport) {
    let total_tok: usize = report.files.iter().map(|f| f.token_count).sum();
    let total_cc: usize = report.files.iter().map(|f| f.complexity_score).sum();
    let n = report.files.len().max(1);
    let clean = report.clean_file_count();
    let dirty = report.files.len() - clean;

    println!("  {} {}", "Files Scanned:".white(), report.files.len());
    println!("  {} {}", "Total Tokens:".white(), format_num(total_tok));
    println!("  {} {}", "Avg Tokens/File:".white(), total_tok / n);
    println!("  {} {}", "Total Complexity:".white(), total_cc);
    println!("  {} {}", "Avg Complexity:".white(), total_cc / n);
    println!();

    let status = if dirty == 0 {
        format!("{clean} files clean").green().to_string()
    } else {
        format!(
            "{} clean, {} with violations",
            clean.to_string().green(),
            dirty.to_string().red()
        )
    };
    println!("  {} {status}", "Status:".white());
    println!(
        "  {} {}",
        "Duration:".white(),
        format_duration(report.duration_ms)
    );
}

fn print_violations(report: &ScanReport) {
    let mut by_law: HashMap<&str, usize> = HashMap::new();
    for file in &report.files {
        for v in &file.violations {
            *by_law.entry(v.law).or_insert(0) += 1;
        }
    }

    println!("{}", "  VIOLATIONS BY TYPE".yellow().bold());
    let mut sorted: Vec<_> = by_law.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    for (law, count) in sorted {
        println!("    {} {}", format!("{count}x").red(), law.dimmed());
    }
}

fn print_offenders(report: &ScanReport) {
    let mut offenders: Vec<_> = report.files.iter().filter(|f| !f.is_clean()).collect();
    if offenders.is_empty() {
        return;
    }

    offenders.sort_by_key(|f| Reverse(f.violation_count()));
    println!();
    println!("{}", "  TOP OFFENDING FILES".yellow().bold());
    for file in offenders.iter().take(5) {
        let name = file
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?");
        let path = format!("({})", file.path.display()).dimmed();
        println!(
            "    {} {} {path}",
            format!("{}x", file.violation_count()).red(),
            name
        );
    }
}

#[allow(clippy::cast_precision_loss)]
fn format_duration(ms: u128) -> String {
    format!("{:.2}s", (ms as f64) / 1000.0)
}

#[allow(clippy::cast_precision_loss)]
fn format_num(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", (n as f64) / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", (n as f64) / 1_000.0)
    } else {
        n.to_string()
    }
}
