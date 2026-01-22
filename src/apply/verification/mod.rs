// src/apply/verification.rs
//! Verification pipeline orchestration.

use crate::analysis::RuleEngine;
use crate::apply::process_runner::CommandRunner;
use crate::apply::types::ApplyContext;
use crate::cli::locality;
use crate::config::Config;
use crate::discovery;
use crate::events::{EventKind, EventLogger};
use crate::spinner::{self, SpinnerClient};
use crate::types::{CheckReport, CommandResult, ScanReport};
use anyhow::Result;
use std::env;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

pub mod report_display;

/// Runs the full verification pipeline.
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

    // Compiler check: removed `mut` from `controller` declaration
    let (client, controller) = if ctx.silent {
        (None, None)
    } else {
        let (cli, ctrl) = spinner::start("slopchop check");
        (Some(cli), Some(ctrl))
    };

    let (mut cmd_results, mut passed) = run_external_commands(
        ctx,
        working_dir,
        &runner,
        &logger,
        client.as_ref(),
        &mut current_step,
        total_steps,
    )?;

    current_step += 1;
    if let Some(c) = &client {
        c.set_macro_step(current_step, total_steps, "Structural Analysis");
    }
    let scan = run_internal_scan(working_dir, client.as_ref())?;
    if scan.has_errors() {
        passed = false;
        logger.log(EventKind::CheckFailed { exit_code: 1 });
    }

    let locality_res = run_locality_if_enabled(
        ctx,
        working_dir,
        client.as_ref(),
        &mut current_step,
        total_steps,
        &logger,
    )?;
    if let Some(ref loc) = locality_res {
        if loc.exit_code != 0 {
            passed = false;
        }
        cmd_results.push(loc.clone());
    }

    // We need to consume controller to stop it.
    // If controller was not mut, we can still move it out of Option.
    if let Some(mut c) = controller {
        c.stop(passed);
    }

    if !ctx.silent {
        crate::apply::advisory::maybe_print_edit_advisory(&ctx.repo_root);
        report_display::print_report(&cmd_results, &scan, locality_res.as_ref());
    }

    if passed {
        logger.log(EventKind::CheckPassed);
    }
    crate::apply::report_writer::write_check_report(&scan, &cmd_results, passed, &ctx.repo_root)?;

    Ok(CheckReport {
        scan,
        commands: cmd_results,
        passed,
    })
}

fn calculate_steps(ctx: &ApplyContext) -> (usize, usize) {
    let cmds = ctx.config.commands.get("check").map_or(0, Vec::len);
    let locality = usize::from(ctx.config.rules.locality.is_enabled());
    (cmds + 1 + locality, cmds)
}

fn run_external_commands(
    ctx: &ApplyContext,
    cwd: &Path,
    runner: &CommandRunner,
    logger: &EventLogger,
    client: Option<&SpinnerClient>,
    step: &mut usize,
    total: usize,
) -> Result<(Vec<CommandResult>, bool)> {
    let mut results = Vec::new();
    let mut passed = true;
    if let Some(commands) = ctx.config.commands.get("check") {
        for cmd in commands {
            *step += 1;
            let label = extract_label(cmd);
            if let Some(c) = client {
                c.set_macro_step(*step, total, label);
            }
            let result = runner.run(cmd, cwd, client)?;
            if result.exit_code != 0 {
                passed = false;
                logger.log(EventKind::CheckFailed {
                    exit_code: result.exit_code,
                });
            }
            results.push(result);
        }
    }
    Ok((results, passed))
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

fn run_internal_scan(cwd: &Path, client: Option<&SpinnerClient>) -> Result<ScanReport> {
    let start = Instant::now();
    let original = env::current_dir()?;
    env::set_current_dir(cwd)?;

    let config = Config::load();
    let files = discovery::discover(&config)?;
    let engine = RuleEngine::new(config);
    let total = files.len();
    let counter = AtomicUsize::new(0);

    let report = engine.scan_with_progress(
        &files,
        &|path| {
            if let Some(c) = client {
                let i = counter.fetch_add(1, Ordering::Relaxed) + 1;
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
                c.step_micro_progress(i, total, format!("Scanning {name}"));
                c.push_log(&format!("{}", path.display()));
            }
        },
        &|status| {
            if let Some(c) = client {
                c.set_micro_status(status);
            }
        },
    );

    let mut final_report = report;
    final_report.duration_ms = start.elapsed().as_millis();
    env::set_current_dir(original)?;
    Ok(final_report)
}

fn run_locality_if_enabled(
    ctx: &ApplyContext,
    cwd: &Path,
    client: Option<&SpinnerClient>,
    step: &mut usize,
    total: usize,
    logger: &EventLogger,
) -> Result<Option<CommandResult>> {
    if !ctx.config.rules.locality.is_enabled() {
        return Ok(None);
    }
    *step += 1;
    if let Some(c) = client {
        c.set_macro_step(*step, total, "Locality Analysis");
    }

    let start = Instant::now();
    let original = env::current_dir()?;
    env::set_current_dir(cwd)?;
    if let Some(c) = client {
        c.set_micro_status("Building dependency graph...");
    }

    let (passed, violations) = locality::check_locality_silent(cwd)?;
    env::set_current_dir(original)?;

    if !passed {
        logger.log(EventKind::CheckFailed { exit_code: 1 });
    }

    #[allow(clippy::cast_possible_truncation)]
    Ok(Some(CommandResult {
        command: "slopchop scan --locality".to_string(),
        exit_code: i32::from(!passed),
        stdout: if passed {
            String::new()
        } else {
            format!("{violations} violations")
        },
        stderr: String::new(),
        duration_ms: start.elapsed().as_millis() as u64,
    }))
}