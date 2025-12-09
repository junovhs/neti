// src/cli/audit.rs
//! CLI handlers for the consolidation audit command.
//! All CLI args are passed via structure to avoid high arity.

use crate::audit::{self, AuditOptions, AuditReport};

use anyhow::Result;
use colored::Colorize;

/// Options for the audit CLI handler.
#[allow(clippy::struct_excessive_bools)]
pub struct AuditCliOptions<'a> {
    pub format: &'a str,
    pub no_dead: bool,
    pub no_dups: bool,
    pub no_patterns: bool,
    pub min_lines: usize,
    pub max: usize,
    pub verbose: bool,
}


/// Runs the consolidation audit with the given options.
///
/// # Errors
/// Returns error if audit fails.
pub fn handle(opts: &AuditCliOptions<'_>) -> Result<()> {
    run_audit(opts)
}

fn run_audit(cli_opts: &AuditCliOptions<'_>) -> Result<()> {
    print_audit_header(cli_opts);

    let options = AuditOptions {
        detect_dead_code: !cli_opts.no_dead,
        detect_duplicates: !cli_opts.no_dups,
        detect_patterns: !cli_opts.no_patterns,
        min_unit_lines: cli_opts.min_lines,
        format: cli_opts.format.to_string(),
        max_opportunities: cli_opts.max,
    };

    let report = audit::run(&options)?;
    let output = audit::format_report(&report, cli_opts.format);

    println!("{output}");

    handle_report_output(&report, cli_opts);

    Ok(())
}

fn print_audit_header(cli_opts: &AuditCliOptions<'_>) {
    if cli_opts.verbose {
        println!("{}", "🔍 Starting consolidation audit...".cyan());
        println!("   Dead code detection: {}", enabled_str(!cli_opts.no_dead));
        println!("   Duplicate detection: {}", enabled_str(!cli_opts.no_dups));
        println!(
            "   Pattern detection:   {}",
            enabled_str(!cli_opts.no_patterns)
        );
        println!("   Min unit size:       {} lines", cli_opts.min_lines);
        println!("   Max opportunities:   {}", cli_opts.max);
        println!();
    } else {
        println!("{}", "🔍 Running consolidation audit...".cyan());
    }
}

fn handle_report_output(report: &AuditReport, cli_opts: &AuditCliOptions<'_>) {
    // For terminal output, also copy AI-friendly version to clipboard
    if cli_opts.format == "terminal" && !report.opportunities.is_empty() {
        let ai_version = audit::report::format_ai_prompt(report);
        match crate::clipboard::copy_to_clipboard(&ai_version) {
            Ok(()) => {
                println!("{}", "✓ AI-friendly summary copied to clipboard".green());
            }
            Err(e) => {
                if cli_opts.verbose {
                    eprintln!(
                        "{}",
                        format!("Note: Could not copy to clipboard: {e}").dimmed()
                    );
                }
            }
        }
    }

    // Exit with non-zero if opportunities found (for CI integration)
    if !report.opportunities.is_empty() && cli_opts.format != "json" {
        println!(
            "\n{}",
            format!(
                "Found {} consolidation opportunities. \
                 Run with --format=json for machine-readable output.",
                report.opportunities.len()
            )
            .dimmed()
        );
    }
}

fn enabled_str(enabled: bool) -> colored::ColoredString {
    if enabled {
        "enabled".green()
    } else {
        "disabled".dimmed()
    }
}
