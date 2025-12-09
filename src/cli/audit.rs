// src/cli/audit.rs
//! CLI handlers for the consolidation audit command.

use crate::audit::{self, AuditOptions};
use anyhow::Result;
use colored::Colorize;

/// Runs the consolidation audit with the given options.
///
/// # Errors
/// Returns error if audit fails.
pub fn handle(
    format: &str,
    no_dead: bool,
    no_dups: bool,
    no_patterns: bool,
    min_lines: usize,
    max: usize,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("{}", "🔍 Starting consolidation audit...".cyan());
        println!("   Dead code detection: {}", enabled_str(!no_dead));
        println!("   Duplicate detection: {}", enabled_str(!no_dups));
        println!("   Pattern detection:   {}", enabled_str(!no_patterns));
        println!("   Min unit size:       {} lines", min_lines);
        println!("   Max opportunities:   {}", max);
        println!();
    } else {
        println!("{}", "🔍 Running consolidation audit...".cyan());
    }

    let options = AuditOptions {
        detect_dead_code: !no_dead,
        detect_duplicates: !no_dups,
        detect_patterns: !no_patterns,
        min_unit_lines: min_lines,
        format: format.to_string(),
        max_opportunities: max,
    };

    let report = audit::run(&options)?;
    let output = audit::format_report(&report, format);

    println!("{output}");

    // For terminal output, also copy AI-friendly version to clipboard
    if format == "terminal" && !report.opportunities.is_empty() {
        let ai_version = audit::report::format_ai_prompt(&report);
        match crate::clipboard::copy_to_clipboard(&ai_version) {
            Ok(()) => {
                println!("{}", "✓ AI-friendly summary copied to clipboard".green());
            }
            Err(e) => {
                if verbose {
                    eprintln!(
                        "{}",
                        format!("Note: Could not copy to clipboard: {e}").dimmed()
                    );
                }
            }
        }
    }

    // Exit with non-zero if opportunities found (for CI integration)
    if !report.opportunities.is_empty() && format != "json" {
        println!(
            "\n{}",
            format!(
                "Found {} consolidation opportunities. Run with --format=json for machine-readable output.",
                report.opportunities.len()
            ).dimmed()
        );
    }

    Ok(())
}

fn enabled_str(enabled: bool) -> colored::ColoredString {
    if enabled {
        "enabled".green()
    } else {
        "disabled".dimmed()
    }
}

