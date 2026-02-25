// src/cli/handlers/mod.rs
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
        return Ok(if report.has_errors() { NetiExit::CheckFailed } else { NetiExit::Success });
    }

    let (client, mut controller) = spinner::start("neti scan");
    let files = discovery::discover(&config)?;
    let total = files.len();
    let engine = Engine::new(config);
    let counter = AtomicUsize::new(0);

    let report = engine.scan_with_progress(
        &files,
        &|path| {
            let i = counter.fetch_add(1, Ordering::Relaxed) + 1;
            client.step_micro_progress(i, total, format!("Scanning {}", path.file_name().and_then(|n| n.to_str()).unwrap_or("file")));
            client.push_log(&format!("{}", path.display()));
        },
        &|status| { client.set_micro_status(status); },
    );

    let has_errors = report.has_errors();
    controller.stop(!has_errors);

    scan_report::print(&report);
    if has_errors {
        reporting::print_report(&report)?;
    }

    Ok(if has_errors { NetiExit::CheckFailed } else { NetiExit::Success })
}

/// Handles the check command. Master pipeline: Scan -> Commands.
pub fn handle_check(json: bool) -> Result<NetiExit> {
    let repo_root = get_repo_root();
    let config = Config::load();

    if json {
        let files = discovery::discover(&config)?;
        let engine = Engine::new(config.clone());
        let scan_report = engine.scan(&files);
        let verif_report = verification::run(&repo_root, |_, _, _| {});
        let passed = !scan_report.has_errors() && verif_report.passed;

        let mut report_text = String::new();
        report_text.push_str("========================================\nNETI SCAN REPORT\n========================================\n\n");
        report_text.push_str(&scan_report::build_summary_string(&scan_report));
        report_text.push('\n');
        if scan_report.has_errors() {
            if let Ok(rich_str) = reporting::build_rich_report(&scan_report) { report_text.push_str(&rich_str); }
        }
        report_text.push_str("\n\n========================================\nNETI COMMANDS REPORT\n========================================\n\n");
        for cmd in &verif_report.commands {
            if cmd.passed { report_text.push_str(&format!("$ {}\n> PASS ({}ms)\n\n", cmd.command, cmd.duration_ms)); }
            else { report_text.push_str(&format!("$ {}\n> FAIL ({}ms)\n{}\n\n", cmd.command, cmd.duration_ms, cmd.output.trim())); }
        }

        std::fs::write("neti-report.txt", &report_text)?;
        return Ok(if passed { NetiExit::Success } else { NetiExit::CheckFailed });
    }

    let (client, mut controller) = spinner::start("neti check");

    client.set_macro_step(1, 2, "Static Analysis");
    let files = discovery::discover(&config)?;
    let engine = Engine::new(config.clone());
    let counter = AtomicUsize::new(0);
    let scan_report = engine.scan_with_progress(
        &files,
        &|path| {
            let i = counter.fetch_add(1, Ordering::Relaxed) + 1;
            client.step_micro_progress(i, files.len(), format!("Scanning {}", path.file_name().and_then(|n| n.to_str()).unwrap_or("file")));
        },
        &|status| { client.set_micro_status(status); },
    );

    client.set_macro_step(2, 2, "Verification Commands");
    let verif_report = verification::run(&repo_root, |cmd, current, total| {
        client.step_micro_progress(current, total, format!("Running: {cmd}"));
    });

    let passed = !scan_report.has_errors() && verif_report.passed;
    controller.stop(passed);

    // Write neti-report.txt (The high-detail "Manual")
    let mut report_text = String::new();
    report_text.push_str("========================================\nNETI SCAN REPORT\n========================================\n\n");
    report_text.push_str(&scan_report::build_summary_string(&scan_report));
    report_text.push('\n');
    if scan_report.has_errors() {
        if let Ok(rich_str) = reporting::build_rich_report(&scan_report) { report_text.push_str(&rich_str); }
    }
    report_text.push_str("\n\n========================================\nNETI COMMANDS REPORT\n========================================\n\n");
    for cmd in &verif_report.commands {
        if cmd.passed { report_text.push_str(&format!("$ {}\n> PASS ({}ms)\n\n", cmd.command, cmd.duration_ms)); }
        else { report_text.push_str(&format!("$ {}\n> FAIL ({}ms)\n{}\n\n", cmd.command, cmd.duration_ms, cmd.output.trim())); }
    }
    std::fs::write("neti-report.txt", &report_text)?;

    // TERMINAL OUTPUT (Succinct Scoreboard)
    scan_report::print(&scan_report);
    print_check_scorecard(&verif_report);

    Ok(if passed { NetiExit::Success } else { NetiExit::CheckFailed })
}

fn print_check_scorecard(report: &verification::VerificationReport) {
    println!("{}", "COMMANDS REPORT".cyan().bold());
    println!("{}", "========================================".dimmed());
    for cmd in &report.commands {
        println!("$ {}", cmd.command.white());
        if cmd.passed { println!("> {} ({}ms)\n", "PASS".green(), cmd.duration_ms); }
        else { println!("> {} ({}ms)\n", "FAIL".red(), cmd.duration_ms); }
    }
    if report.passed { println!("{} {} commands passed.", "✓".green().bold(), report.total_commands()); }
    else { println!("{} {}/{} failed. Details in {}.", "✗".red().bold(), report.failed_count(), report.total_commands(), "neti-report.txt".yellow()); }
}
