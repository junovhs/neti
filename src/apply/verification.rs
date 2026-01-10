// src/apply/verification.rs
use crate::apply::process_runner::CommandRunner;
use crate::apply::types::ApplyContext;
use crate::cli::locality;
use crate::config::Config;
use crate::discovery;
use crate::analysis::RuleEngine;
use crate::events::{EventKind, EventLogger};
use crate::reporting;
use crate::spinner::Spinner;
use crate::types::{CheckReport, CommandResult, ScanReport};
use anyhow::Result;
use colored::Colorize;
use std::env;
use std::path::Path;
use std::time::Instant;

/// Runs the full verification pipeline: External Commands -> Scan -> Locality.
///
/// # Errors
/// Returns error if command execution fails.
pub fn run_verification_pipeline<P: AsRef<Path>>(
    ctx: &ApplyContext,
    cwd: P,
) -> Result<CheckReport> {
    let logger = EventLogger::new(&ctx.repo_root);
    logger.log(EventKind::CheckStarted);

    if !ctx.silent {
        println!("{}", "\n  Verifying changes...".blue().bold());
    }

    let working_dir = cwd.as_ref();
    let runner = CommandRunner::new(ctx.silent);

    // 1. External Commands
    let (mut command_results, mut passed) = run_external_checks(ctx, working_dir, &runner, &logger)?;

    // 2. Internal Scan
    let scan_report = run_internal_scan(working_dir, ctx.silent)?;
    if scan_report.has_errors() {
        passed = false;
        logger.log(EventKind::CheckFailed { exit_code: 1 });
    }

    // 3. Locality Scan
    let locality_result = run_locality_scan(working_dir, ctx.silent)?;
    if locality_result.exit_code != 0 {
        passed = false;
        logger.log(EventKind::CheckFailed { exit_code: 1 });
    }
    command_results.push(locality_result);

    // 4. Heuristic Nag (advisory for high edit volume)
    if !ctx.silent {
        crate::apply::advisory::maybe_print_edit_advisory(&ctx.repo_root);
    }

    if passed {
        logger.log(EventKind::CheckPassed);
    }

    Ok(CheckReport {
        scan: scan_report,
        commands: command_results,
        passed,
    })
}

fn run_external_checks(
    ctx: &ApplyContext,
    cwd: &Path,
    runner: &CommandRunner,
    logger: &EventLogger
) -> Result<(Vec<CommandResult>, bool)> {
    let mut results = Vec::new();
    let mut passed = true;

    if let Some(commands) = ctx.config.commands.get("check") {
        for cmd in commands {
            let result = runner.run(cmd, cwd)?;
            if result.exit_code != 0 {
                passed = false;
                logger.log(EventKind::CheckFailed { exit_code: result.exit_code });
            }
            results.push(result);
        }
    }
    Ok((results, passed))
}

/// Runs verification using the repo root.
///
/// # Errors
/// Returns error if command execution fails.
pub fn run_verification_auto(ctx: &ApplyContext) -> Result<bool> {
    let report = run_verification_pipeline(ctx, &ctx.repo_root)?;
    Ok(report.passed)
}

fn run_internal_scan(cwd: &Path, silent: bool) -> Result<ScanReport> {
    let sp = if silent { None } else { Some(Spinner::start("slopchop scan")) };
    let start = Instant::now();

    let original_cwd = env::current_dir()?;
    env::set_current_dir(cwd)?;

    let config = Config::load();
    let files = discovery::discover(&config)?;
    let engine = RuleEngine::new(config);
    let mut report = engine.scan(&files);
    report.duration_ms = start.elapsed().as_millis();

    env::set_current_dir(original_cwd)?;

    let success = !report.has_errors();
    if let Some(s) = sp { s.stop(success); }

    if !success && !silent {
        reporting::print_report(&report)?;
    }

    Ok(report)
}

fn run_locality_scan(cwd: &Path, silent: bool) -> Result<CommandResult> {
    let config = Config::load();

    if !config.rules.locality.is_enabled() || !config.rules.locality.is_error_mode() {
        return Ok(CommandResult {
            command: "slopchop scan --locality".to_string(),
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: 0,
        });
    }

    let start = Instant::now();
    let sp = if silent { None } else { Some(Spinner::start("slopchop scan --locality")) };

    let passed: bool;
    let output: String;

    if silent {
        let original_cwd = env::current_dir()?;
        env::set_current_dir(cwd)?;
        let (p, v) = locality::check_locality_silent(cwd)?;
        env::set_current_dir(original_cwd)?;
        passed = p;
        if passed {
            output = String::new();
        } else {
            output = format!("Locality check failed with {v} violations.");
        }
    } else {
        let original_cwd = env::current_dir()?;
        env::set_current_dir(cwd)?;
        let res = locality::run_locality_check(cwd)?;
        env::set_current_dir(original_cwd)?;
        passed = res.passed;
        if passed {
            output = String::new();
        } else {
            output = format!("Locality check failed with {} violations.", res.violations);
        }
    }

    let duration = start.elapsed();
    if let Some(s) = sp { s.stop(passed); }

    #[allow(clippy::cast_possible_truncation)]
    Ok(CommandResult {
        command: "slopchop scan --locality".to_string(),
        exit_code: i32::from(!passed),
        stdout: output,
        stderr: String::new(),
        duration_ms: duration.as_millis() as u64,
    })
}