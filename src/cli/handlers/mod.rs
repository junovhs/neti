// src/cli/handlers/mod.rs
//! Core analysis command handlers.

use crate::analysis::Engine;
use crate::config::Config;
use crate::discovery;
use crate::exit::NetiExit;
use crate::reporting;
use crate::spinner;
use crate::types::CheckReport;
use crate::verification;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

mod check_report;
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
        let report = Engine::scan(&config, &files);
        reporting::print_json(&report)?;
        return Ok(if report.has_errors() {
            NetiExit::CheckFailed
        } else {
            NetiExit::Success
        });
    }

    let (client, mut controller) = spinner::start("neti scan");
    let files = discovery::discover(&config)?;
    let total = files.len();
    let counter = AtomicUsize::new(0);

    let report = Engine::scan_with_progress(
        &config,
        &files,
        &|path| {
            let i = counter.fetch_add(1, Ordering::Relaxed) + 1;
            client.step_micro_progress(
                i,
                total,
                format!(
                    "Scanning {}",
                    path.file_name().and_then(|n| n.to_str()).unwrap_or("file")
                ),
            );
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

/// Handles the check command. Master pipeline: Scan -> Locality -> Commands.
pub fn handle_check(json: bool) -> Result<NetiExit> {
    let repo_root = get_repo_root();
    let config = Config::load();

    if json {
        return handle_check_json(&repo_root, &config);
    }

    handle_check_interactive(&repo_root, &config)
}

/// JSON mode: emit `CheckReport` to stdout, write `neti-report.txt`.
fn handle_check_json(repo_root: &Path, config: &Config) -> Result<NetiExit> {
    let files = discovery::discover(config)?;
    let scan_report = Engine::scan(config, &files);
    let locality_report = super::locality::check_locality_silent(repo_root, config)?;
    let verif_report = verification::run(repo_root, |_, _, _| {});

    let passed = !scan_report.has_errors() && locality_report.passed && verif_report.passed;

    let text = check_report::build_report_text(&scan_report, &verif_report, Some(&locality_report));
    std::fs::write("neti-report.txt", &text)?;

    let check_report = CheckReport {
        scan: scan_report,
        commands: verif_report.commands,
        locality: Some(locality_report),
        passed,
    };
    reporting::print_json(&check_report)?;

    Ok(if passed {
        NetiExit::Success
    } else {
        NetiExit::CheckFailed
    })
}

/// Interactive mode: spinner UI, write `neti-report.txt`, print scorecard.
fn handle_check_interactive(repo_root: &Path, config: &Config) -> Result<NetiExit> {
    let (client, mut controller) = spinner::start("neti check");

    client.set_macro_step(1, 3, "Static Analysis");
    let files = discovery::discover(config)?;
    let counter = AtomicUsize::new(0);
    let file_count = files.len();
    let scan_report = Engine::scan_with_progress(
        config,
        &files,
        &|path| {
            let i = counter.fetch_add(1, Ordering::Relaxed) + 1;
            client.step_micro_progress(
                i,
                file_count,
                format!(
                    "Scanning {}",
                    path.file_name().and_then(|n| n.to_str()).unwrap_or("file")
                ),
            );
        },
        &|status| {
            client.set_micro_status(status);
        },
    );

    client.set_macro_step(2, 3, "Law of Locality");
    let locality_report = super::locality::check_locality_silent(repo_root, config)?;

    client.set_macro_step(3, 3, "Verification Commands");
    let verif_report = verification::run(repo_root, |cmd, current, total| {
        client.step_micro_progress(current, total, format!("Running: {cmd}"));
    });

    let passed = !scan_report.has_errors() && locality_report.passed && verif_report.passed;
    controller.stop(passed);

    let text = check_report::build_report_text(&scan_report, &verif_report, Some(&locality_report));
    std::fs::write("neti-report.txt", &text)?;

    scan_report::print(&scan_report);
    check_report::print_locality_scorecard(&locality_report);
    check_report::print_commands_scorecard(&verif_report);

    Ok(if passed {
        NetiExit::Success
    } else {
        NetiExit::CheckFailed
    })
}
