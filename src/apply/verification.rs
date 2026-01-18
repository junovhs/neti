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
use crate::types::{CheckReport, CommandResult, ScanReport, FileReport};
use anyhow::Result;
use colored::Colorize;
use std::env;
use std::fmt::Write;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// Runs the full verification pipeline: External Commands -> Scan -> Locality.
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

    let (mut command_results, mut passed) = run_external_checks(ctx, working_dir, &runner, &logger)?;

    let scan_report = run_internal_scan(working_dir, ctx.silent)?;
    if scan_report.has_errors() {
        passed = false;
        logger.log(EventKind::CheckFailed { exit_code: 1 });
    }

    let locality_result = run_locality_scan(working_dir, ctx.silent)?;
    if locality_result.exit_code != 0 {
        passed = false;
        logger.log(EventKind::CheckFailed { exit_code: 1 });
    }
    command_results.push(locality_result);

    if !ctx.silent {
        crate::apply::advisory::maybe_print_edit_advisory(&ctx.repo_root);
    }

    if passed {
        logger.log(EventKind::CheckPassed);
    }

    write_check_report(&scan_report, &command_results, passed, &ctx.repo_root)?;

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

fn run_internal_scan(cwd: &Path, silent: bool) -> Result<ScanReport> {
    let sp = if silent { None } else { Some(Spinner::start("slopchop scan")) };
    let start = Instant::now();

    let original_cwd = env::current_dir()?;
    env::set_current_dir(cwd)?;

    let config = Config::load();
    let files = discovery::discover(&config)?;
    let engine = RuleEngine::new(config);

    // Progress throttling: update spinner every 5 files to avoid lock contention
    let counter = AtomicUsize::new(0);
    
    let report = engine.scan_with_progress(&files, &|path| {
        if let Some(s) = &sp {
            let i = counter.fetch_add(1, Ordering::Relaxed);
            if i % 5 == 0 {
                s.push_log(&format!("Scanning {}", path.display()));
            }
        }
    });
    
    // Explicitly set duration since engine doesn't track wall time of its internal phases
    let mut final_report = report;
    final_report.duration_ms = start.elapsed().as_millis();

    env::set_current_dir(original_cwd)?;

    let success = !final_report.has_errors();
    if let Some(s) = sp { s.stop(success); }

    if !success && !silent {
        reporting::print_report(&final_report)?;
    }

    Ok(final_report)
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

// REPORT GENERATION
fn write_check_report(scan: &ScanReport, cmds: &[CommandResult], passed: bool, root: &Path) -> Result<()> {
    let mut out = String::with_capacity(10000);
    
    write_header(&mut out, passed)?;
    write_dashboard(&mut out, &scan.files)?;
    write_violations(&mut out, scan, cmds, passed)?;
    write_full_logs(&mut out, cmds)?;

    std::fs::write(root.join("slopchop-report.txt"), out)?;
    Ok(())
}

fn write_header(out: &mut String, passed: bool) -> Result<()> {
    writeln!(out, "SLOPCHOP CHECK REPORT")?;
    writeln!(out, "Generated: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"))?;
    writeln!(out, "Status: {}\n", if passed { "PASSED" } else { "FAILED" })?;
    Ok(())
}

fn write_dashboard(out: &mut String, files: &[FileReport]) -> Result<()> {
    writeln!(out, "=== DASHBOARD ===")?;
    let mut files_sorted = files.to_vec();
    
    writeln!(out, "Top 5 Cognitive Complexity:")?;
    files_sorted.sort_by(|a, b| b.complexity_score.cmp(&a.complexity_score));
    for f in files_sorted.iter().take(5) {
        writeln!(out, "  {:<4} {}", f.complexity_score, f.path.display())?;
    }
    
    writeln!(out, "\nTop 5 Largest Files (Tokens):")?;
    files_sorted.sort_by(|a, b| b.token_count.cmp(&a.token_count));
    for f in files_sorted.iter().take(5) {
        writeln!(out, "  {:<5} {}", f.token_count, f.path.display())?;
    }
    Ok(())
}

fn write_violations(out: &mut String, scan: &ScanReport, cmds: &[CommandResult], passed: bool) -> Result<()> {
    if passed {
        writeln!(out, "\n=== VIOLATIONS ===\nNone. Codebase is clean.")?;
        return Ok(());
    }

    writeln!(out, "\n=== VIOLATIONS ===")?;
    if scan.has_errors() {
        writeln!(out, "[SlopChop Internal Rules]")?;
        for file in scan.files.iter().filter(|f| !f.is_clean()) {
            for v in &file.violations {
                writeln!(out, "{}:{} | {} | {}", file.path.display(), v.row, v.law, v.message)?;
            }
        }
    }
    
    writeln!(out, "\n[External Tools]")?;
    for cmd in cmds {
        if cmd.exit_code != 0 {
            writeln!(out, "FAILED: {} (Exit Code: {})", cmd.command, cmd.exit_code)?;
            writeln!(out, "-- STDOUT --\n{}", cmd.stdout)?;
            writeln!(out, "-- STDERR --\n{}", cmd.stderr)?;
        }
    }
    Ok(())
}

fn write_full_logs(out: &mut String, cmds: &[CommandResult]) -> Result<()> {
    writeln!(out, "\n=== FULL OUTPUT LOGS ===")?;
    for cmd in cmds {
        writeln!(out, "\n>>> COMMAND: {}", cmd.command)?;
        if !cmd.stdout.is_empty() { writeln!(out, "{}", cmd.stdout)?; }
        if !cmd.stderr.is_empty() { writeln!(out, "{}", cmd.stderr)?; }
    }
    Ok(())
}