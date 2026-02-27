// src/cli/handlers/check_report.rs
//! Report building and scorecard display for `neti check`.

use crate::types::{CommandResult, LocalityReport, ScanReport};
use crate::verification::VerificationReport;
use crate::{cli::handlers::scan_report, reporting};
use colored::Colorize;

/// Builds the plain-text report written to `neti-report.txt`.
pub fn build_report_text(
    scan_report: &ScanReport,
    verif_report: &VerificationReport,
    locality_report: Option<&LocalityReport>,
) -> String {
    let mut out = String::new();
    out.push_str(
        "========================================\n\
         NETI SCAN REPORT\n\
         ========================================\n\n",
    );
    out.push_str(&scan_report::build_summary_string(scan_report));
    out.push('\n');

    if scan_report.has_errors() {
        if let Ok(rich_str) = reporting::build_rich_report(scan_report) {
            out.push_str(&rich_str);
        }
    }

    if let Some(loc) = locality_report {
        out.push_str(
            "\n\n========================================\n\
             NETI LOCALITY REPORT\n\
             ========================================\n\n",
        );
        append_locality_result(&mut out, loc);
    }

    out.push_str(
        "\n\n========================================\n\
         NETI COMMANDS REPORT\n\
         ========================================\n\n",
    );
    for cmd in &verif_report.commands {
        append_command_result(&mut out, cmd);
    }
    out
}

/// Appends locality results to the report text.
fn append_locality_result(out: &mut String, loc: &LocalityReport) {
    out.push_str(&format!("Mode: {}\n", loc.mode));
    out.push_str(&format!("Edges analyzed: {}\n", loc.total_edges));
    out.push_str(&format!("Violations: {}\n", loc.violation_count));
    out.push_str(&format!("Cycles: {}\n", loc.cycle_count));

    for v in &loc.violations {
        out.push_str(&format!(
            "  {} → {} (distance: {}, target: {})\n",
            v.from.display(),
            v.to.display(),
            v.distance,
            v.target_role,
        ));
    }

    for (i, cycle) in loc.cycles.iter().enumerate() {
        let paths: Vec<String> = cycle.iter().map(|p| p.display().to_string()).collect();
        out.push_str(&format!("  cycle {}: {}\n", i + 1, paths.join(" → ")));
    }

    if loc.passed {
        out.push_str("Result: PASS\n");
    } else {
        out.push_str("Result: FAIL\n");
    }
}

/// Appends a single command result to the report text.
fn append_command_result(out: &mut String, cmd: &CommandResult) {
    if cmd.passed() {
        out.push_str(&format!(
            "$ {}\n> PASS ({}ms)\n\n",
            cmd.command(),
            cmd.duration_ms()
        ));
    } else {
        out.push_str(&format!(
            "$ {}\n> FAIL ({}ms)\n{}\n\n",
            cmd.command(),
            cmd.duration_ms(),
            cmd.output().trim()
        ));
    }
}

/// Prints the locality section of the check scorecard.
pub fn print_locality_scorecard(loc: &LocalityReport) {
    println!("{}", "LOCALITY REPORT".cyan().bold());
    println!("{}", "========================================".dimmed());
    println!("Mode: {}", loc.mode.white());
    println!("Edges: {}", loc.total_edges);

    if loc.violation_count == 0 && loc.cycle_count == 0 {
        println!("{} No locality violations.", "✓".green().bold());
    } else {
        for v in &loc.violations {
            let msg = format!(
                "{} → {} (D={}, {})",
                v.from.display(),
                v.to.display(),
                v.distance,
                v.target_role,
            );
            if loc.mode == "error" {
                println!("  {} {}", "✗".red().bold(), msg.red());
            } else {
                println!("  {} {}", "~".yellow().bold(), msg.yellow());
            }
        }
        if loc.cycle_count > 0 {
            println!("{} {} dependency cycles", "✗".red().bold(), loc.cycle_count);
        }
    }
    println!();
}

/// Prints the commands section of the check scorecard.
pub fn print_commands_scorecard(report: &VerificationReport) {
    println!("{}", "COMMANDS REPORT".cyan().bold());
    println!("{}", "========================================".dimmed());
    for cmd in &report.commands {
        println!("$ {}", cmd.command().white());
        if cmd.passed() {
            println!("> {} ({}ms)\n", "PASS".green(), cmd.duration_ms());
        } else {
            println!("> {} ({}ms)\n", "FAIL".red(), cmd.duration_ms());
        }
    }
    if report.passed {
        println!(
            "{} {} commands passed.",
            "✓".green().bold(),
            report.total_commands()
        );
    } else {
        println!(
            "{} {}/{} failed. Details in {}.",
            "✗".red().bold(),
            report.failed_count(),
            report.total_commands(),
            "neti-report.txt".yellow()
        );
    }
}
