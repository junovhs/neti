//! Core analysis command handlers.

use crate::analysis::Engine;
use crate::config::Config;
use crate::discovery;
use crate::exit::NetiExit;
use crate::reporting;
use crate::spinner;
use crate::verification;
use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

pub mod scan_report;

#[must_use]
pub fn get_repo_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Handles the scan command.
///
/// # Errors
/// Returns error if scan execution fails.
pub fn handle_scan(verbose: bool, locality: bool, json: bool) -> Result<NetiExit> {
    if locality {
        return super::locality::handle_locality();
    }

    let mut config = Config::load();
    config.verbose = verbose;

    if json {
        let files = discovery::discover(&config)?;
        let engine = Engine::new(config);
        let report = engine.scan(&files);
        reporting::print_json(&report)?;
        return Ok(if report.has_errors() {
            NetiExit::CheckFailed
        } else {
            NetiExit::Success
        });
    }

    let (client, mut controller) = spinner::start("neti scan");
    client.set_micro_status("Discovering files...");

    let files = discovery::discover(&config)?;
    let total = files.len();
    let engine = Engine::new(config);
    let counter = AtomicUsize::new(0);

    let report = engine.scan_with_progress(
        &files,
        &|path| {
            let i = counter.fetch_add(1, Ordering::Relaxed) + 1;
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
            client.step_micro_progress(i, total, format!("Scanning {name}"));
            client.push_log(&format!("{}", path.display()));
        },
        &|status| {
            client.set_micro_status(status);
        },
    );

    let has_errors = report.has_errors();
    controller.stop(!has_errors);

    scan_report::print(&report);
    if has_errors {
        reporting::print_report(&report)?;
    }

    Ok(if has_errors {
        NetiExit::CheckFailed
    } else {
        NetiExit::Success
    })
}

/// Handles the check command.
///
/// # Errors
/// Returns error if report file cannot be written.
pub fn handle_check(json: bool) -> Result<NetiExit> {
    let repo_root = get_repo_root();

    if json {
        let report = verification::run(&repo_root, |_, _, _| {});
        std::fs::write("neti-report.txt", &report.output)?;
        reporting::print_json(&report)?;
        return Ok(if report.passed {
            NetiExit::Success
        } else {
            NetiExit::CheckFailed
        });
    }

    let (client, mut controller) = spinner::start("neti check");
    client.set_micro_status("Running verification commands...");

    let report = verification::run(&repo_root, |cmd, current, total| {
        client.step_micro_progress(current, total, format!("Running: {cmd}"));
        client.push_log(cmd);
    });

    std::fs::write("neti-report.txt", &report.output)?;

    controller.stop(report.passed);

    print_check_scorecard(&report);

    Ok(if report.passed {
        NetiExit::Success
    } else {
        NetiExit::CheckFailed
    })
}

/// Prints a formatted scorecard for the verification report.
fn print_check_scorecard(report: &verification::VerificationReport) {
    use std::time::Duration;

    #[allow(clippy::cast_possible_truncation)]
    let duration = Duration::from_millis(report.duration_ms);

    println!();
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════".blue()
    );
    println!(
        "{} {}",
        "  SLOPCHOP CHECK REPORT".white().bold(),
        format!("({duration:.2?})").dimmed()
    );
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════".blue()
    );
    println!();

    // Command results
    println!("{}", "  COMMANDS".cyan().bold());
    println!(
        "{}",
        "  ─────────────────────────────────────────────────────────".blue()
    );

    for cmd in &report.commands {
        let status = if cmd.passed {
            "✓".green()
        } else {
            "✗".red()
        };
        let duration_str = format!("{:>4}ms", cmd.duration_ms).dimmed();
        let cmd_display = if cmd.command.len() > 40 {
            format!("{}...", &cmd.command[..37])
        } else {
            cmd.command.clone()
        };

        println!("  {} {} {}", status, cmd_display.white(), duration_str);

        // Show errors/warnings if any
        let errors = cmd.error_count();
        let warnings = cmd.warning_count();

        if errors > 0 || warnings > 0 {
            let mut parts = Vec::new();
            if errors > 0 {
                parts.push(format!("{} {}", errors.to_string().red(), "errors".red()));
            }
            if warnings > 0 {
                parts.push(format!(
                    "{} {}",
                    warnings.to_string().yellow(),
                    "warnings".yellow()
                ));
            }
            println!("      └─ {}", parts.join(", "));
        }
    }

    println!();

    // Summary
    println!("{}", "  SUMMARY".cyan().bold());
    println!(
        "{}",
        "  ─────────────────────────────────────────────────────────".blue()
    );

    let total_cmds = report.total_commands();
    let passed = report.passed_count();
    let failed = report.failed_count();
    let total_errors = report.total_errors();
    let total_warnings = report.total_warnings();

    println!(
        "  Commands:  {} total, {} {}, {} {}",
        total_cmds.to_string().white().bold(),
        passed.to_string().green(),
        "passed".green(),
        failed.to_string().red(),
        "failed".red()
    );

    println!(
        "  Output:    {} {}, {} {}",
        total_errors.to_string().red().bold(),
        if total_errors == 1 { "error" } else { "errors" }.red(),
        total_warnings.to_string().yellow().bold(),
        if total_warnings == 1 {
            "warning"
        } else {
            "warnings"
        }
        .yellow()
    );

    println!();

    // Final verdict
    if report.passed {
        println!("{}", "  ✓ ALL CHECKS PASSED".green().bold());
    } else {
        println!("{}", "  ✗ CHECKS FAILED".red().bold());
        println!();
        println!("  See neti-report.txt for full output.");
    }

    println!();
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════".blue()
    );
}
