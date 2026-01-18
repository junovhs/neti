// src/apply/verification.rs
use crate::apply::process_runner::CommandRunner;
use crate::apply::types::ApplyContext;
use crate::cli::locality;
use crate::config::Config;
use crate::discovery;
use crate::analysis::RuleEngine;
use crate::events::{EventKind, EventLogger};
use crate::spinner::Spinner;
use crate::types::{CheckReport, CommandResult, ScanReport};
use anyhow::Result;
use std::env;
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

    let working_dir = cwd.as_ref();
    let runner = CommandRunner::new(ctx.silent);

    let (total_steps, _) = calculate_steps(ctx);
    let mut current_step = 0;

    let hud = if ctx.silent { None } else { Some(Spinner::start("Verification Pipeline")) };

    // 1. External Checks
    let (mut command_results, mut passed) = run_external_commands(
        ctx,
        working_dir,
        &runner,
        &logger,
        hud.as_ref(),
        &mut current_step,
        total_steps
    )?;

    // 2. Internal Scan
    current_step += 1;
    if let Some(h) = &hud {
        h.set_macro_step(current_step, total_steps, "slopchop scan");
    }

    let scan_report = run_internal_scan(working_dir, hud.as_ref())?;
    if scan_report.has_errors() {
        passed = false;
        logger.log(EventKind::CheckFailed { exit_code: 1 });
    }

    // 3. Locality
    if ctx.config.rules.locality.is_enabled() {
        current_step += 1;
        if let Some(h) = &hud {
            h.set_macro_step(current_step, total_steps, "slopchop scan --locality");
        }
        let loc_res = run_locality_scan(working_dir, hud.as_ref())?;
        if loc_res.exit_code != 0 {
            passed = false;
            logger.log(EventKind::CheckFailed { exit_code: 1 });
        }
        command_results.push(loc_res);
    }

    finalize_pipeline(ctx, passed, &logger, hud.as_ref());

    crate::apply::report_writer::write_check_report(&scan_report, &command_results, passed, &ctx.repo_root)?;

    Ok(CheckReport {
        scan: scan_report,
        commands: command_results,
        passed,
    })
}

fn calculate_steps(ctx: &ApplyContext) -> (usize, usize) {
    let check_cmds = ctx.config.commands.get("check").map_or(0, std::vec::Vec::len);
    let do_locality = ctx.config.rules.locality.is_enabled();
    let total = check_cmds + 1 + usize::from(do_locality);
    (total, check_cmds)
}

fn run_external_commands(
    ctx: &ApplyContext,
    cwd: &Path,
    runner: &CommandRunner,
    logger: &EventLogger,
    hud: Option<&Spinner>,
    current_step: &mut usize,
    total_steps: usize
) -> Result<(Vec<CommandResult>, bool)> {
    let mut results = Vec::new();
    let mut passed = true;

    if let Some(commands) = ctx.config.commands.get("check") {
        for cmd in commands {
            *current_step += 1;
            if let Some(h) = hud {
                h.set_macro_step(*current_step, total_steps, cmd.clone());
            }
            
            let result = runner.run(cmd, cwd, hud)?;
            
            if result.exit_code != 0 {
                passed = false;
                logger.log(EventKind::CheckFailed { exit_code: result.exit_code });
            }
            results.push(result);
        }
    }
    Ok((results, passed))
}

fn finalize_pipeline(ctx: &ApplyContext, passed: bool, logger: &EventLogger, hud: Option<&Spinner>) {
    // Stop the spinner first to ensure UI is clear before printing logs
    if let Some(h) = hud {
        h.stop(passed);
    }

    if !ctx.silent {
        crate::apply::advisory::maybe_print_edit_advisory(&ctx.repo_root);
    }

    if passed {
        logger.log(EventKind::CheckPassed);
    }
}

fn run_internal_scan(cwd: &Path, hud: Option<&Spinner>) -> Result<ScanReport> {
    let start = Instant::now();
    let original_cwd = env::current_dir()?;
    env::set_current_dir(cwd)?;

    let config = Config::load();
    let files = discovery::discover(&config)?;
    let engine = RuleEngine::new(config);
    let total_files = files.len();
    let counter = AtomicUsize::new(0);
    
    let report = engine.scan_with_progress(
        &files, 
        &|path| {
            if let Some(h) = hud {
                let i = counter.fetch_add(1, Ordering::Relaxed) + 1;
                h.step_micro_progress(i, total_files, format!("Scanning {}", path.display()));
            }
        },
        &|status| {
            if let Some(h) = hud {
                h.set_micro_status(status);
            }
        }
    );
    
    let mut final_report = report;
    final_report.duration_ms = start.elapsed().as_millis();

    env::set_current_dir(original_cwd)?;
    Ok(final_report)
}

fn run_locality_scan(cwd: &Path, hud: Option<&Spinner>) -> Result<CommandResult> {
    let start = Instant::now();
    let original_cwd = env::current_dir()?;
    env::set_current_dir(cwd)?;

    if let Some(h) = hud {
        h.set_micro_status("Building dependency graph...");
    }

    let (passed, violations) = locality::check_locality_silent(cwd)?;
    
    env::set_current_dir(original_cwd)?;

    let output = if passed {
        String::new()
    } else {
        format!("Locality check failed with {violations} violations.")
    };

    #[allow(clippy::cast_possible_truncation)]
    Ok(CommandResult {
        command: "slopchop scan --locality".to_string(),
        exit_code: i32::from(!passed),
        stdout: output,
        stderr: String::new(),
        duration_ms: start.elapsed().as_millis() as u64,
    })
}