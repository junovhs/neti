// src/apply/verification/report_display.rs
//! Verification report display formatting.

use crate::types::{CommandResult, ScanReport};
use colored::Colorize;
use std::cmp::Reverse;
use std::collections::HashMap;

/// Prints the comprehensive analysis report.
pub fn print_report(
    commands: &[CommandResult],
    scan: &ScanReport,
    locality: Option<&CommandResult>,
) {
    println!();
    println!("{}", "─".repeat(60).dimmed());
    println!("{}", " VERIFICATION REPORT".bold());
    println!("{}", "─".repeat(60).dimmed());
    println!();

    print_command_results(commands, locality);
    println!();
    println!("{}", "─".repeat(60).dimmed());
    println!("{}", " STRUCTURAL ANALYSIS".bold());
    println!("{}", "─".repeat(60).dimmed());
    println!();
    print_scan_stats(scan);

    if scan.has_errors() {
        println!();
        print_violations(scan);
    }

    println!();
    println!("{}", "─".repeat(60).dimmed());
}

fn print_command_results(commands: &[CommandResult], locality: Option<&CommandResult>) {
    for cmd in commands {
        if cmd.command.contains("--locality") {
            continue;
        }
        let label = extract_label(&cmd.command);
        let status = if cmd.exit_code == 0 {
            "passed".green()
        } else {
            "failed".red()
        };
        let dur = format_duration_u64(cmd.duration_ms);
        println!(
            "  {} {} {}",
            label.to_uppercase().cyan(),
            status,
            dur.dimmed()
        );
    }
    if let Some(loc) = locality {
        let status = if loc.exit_code == 0 {
            "passed".green()
        } else {
            "failed".red()
        };
        let dur = format_duration_u64(loc.duration_ms);
        println!("  {} {} {}", "LOCALITY".cyan(), status, dur.dimmed());
    }
}

fn extract_label(cmd: &str) -> String {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    match parts.as_slice() {
        ["cargo", sub, ..] => format!("cargo {sub}"),
        ["npm", "run", script, ..] => format!("npm {script}"),
        [prog, sub, ..] => format!("{prog} {sub}"),
        [prog] => (*prog).to_string(),
        [] => "command".to_string(),
    }
}

fn print_scan_stats(scan: &ScanReport) {
    let total_tok: usize = scan.files.iter().map(|f| f.token_count).sum();
    let total_cc: usize = scan.files.iter().map(|f| f.complexity_score).sum();
    let n = scan.files.len().max(1);
    let clean = scan.clean_file_count();
    let dirty = scan.files.len() - clean;

    println!("  {} {}", "Files Analyzed:".white(), scan.files.len());
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
        format_duration_u128(scan.duration_ms)
    );
}

fn print_violations(scan: &ScanReport) {
    let mut by_law: HashMap<&str, usize> = HashMap::new();
    for file in &scan.files {
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

    print_top_offenders(scan);
}

fn print_top_offenders(scan: &ScanReport) {
    let mut offenders: Vec<_> = scan.files.iter().filter(|f| !f.is_clean()).collect();
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
fn format_duration_u64(ms: u64) -> String {
    format!("{:.1}s", (ms as f64) / 1000.0)
}

#[allow(clippy::cast_precision_loss)]
fn format_duration_u128(ms: u128) -> String {
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
