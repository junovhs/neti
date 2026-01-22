// src/cli/handlers/mod.rs
//! Core analysis command handlers.

use crate::analysis::RuleEngine;
use crate::apply;
use crate::apply::types::ApplyContext;
use crate::config::Config;
use crate::discovery;
use crate::exit::SlopChopExit;
use crate::map;
use crate::pack::{self, OutputFormat, PackOptions};
use crate::reporting;
use crate::signatures::{self, SignatureOptions};
use crate::spinner;
use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

pub mod scan_report;

#[must_use]
pub fn get_repo_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[derive(Debug, Clone)]
pub struct PackArgs {
    pub stdout: bool,
    pub copy: bool,
    pub noprompt: bool,
    pub format: OutputFormat,
    pub skeleton: bool,
    pub code_only: bool,
    pub verbose: bool,
    pub target: Option<PathBuf>,
    pub focus: Vec<PathBuf>,
    pub depth: usize,
}

/// Handles the scan command.
pub fn handle_scan(verbose: bool, locality: bool, json: bool) -> Result<SlopChopExit> {
    if locality {
        return super::locality::handle_locality();
    }

    let mut config = Config::load();
    config.verbose = verbose;

    if json {
        let files = discovery::discover(&config)?;
        let engine = RuleEngine::new(config);
        let report = engine.scan(&files);
        reporting::print_json(&report)?;
        return Ok(if report.has_errors() {
            SlopChopExit::CheckFailed
        } else {
            SlopChopExit::Success
        });
    }

    let (client, mut controller) = spinner::start("slopchop scan");
    client.set_micro_status("Discovering files...");

    let files = discovery::discover(&config)?;
    let total = files.len();
    let engine = RuleEngine::new(config);
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
        SlopChopExit::CheckFailed
    } else {
        SlopChopExit::Success
    })
}

/// Handles the check command.
pub fn handle_check(json: bool) -> Result<SlopChopExit> {
    let config = Config::load();
    let repo_root = get_repo_root();
    let mut ctx = ApplyContext::new(&config, repo_root.clone());
    ctx.silent = json;

    let report = apply::verification::run_verification_pipeline(&ctx, &repo_root)?;

    if json {
        reporting::print_json(&report)?;
    } else if report.passed {
        println!("{}", "✓ All checks passed.".green().bold());
    }

    Ok(if report.passed {
        SlopChopExit::Success
    } else {
        SlopChopExit::CheckFailed
    })
}

/// Handles the pack command.
pub fn handle_pack(args: PackArgs) -> Result<SlopChopExit> {
    let opts = PackOptions {
        stdout: args.stdout,
        copy: args.copy,
        verbose: args.verbose,
        prompt: !args.noprompt,
        format: args.format,
        skeleton: args.skeleton,
        code_only: args.code_only,
        target: args.target,
        focus: args.focus,
        depth: args.depth,
    };

    if args.stdout {
        pack::run(&opts)?;
        return Ok(SlopChopExit::Success);
    }

    let (client, mut controller) = spinner::start("slopchop pack");
    client.set_micro_status("Discovering files...");

    let res = pack::run_with_progress(&opts, |done, total, msg| {
        client.step_micro_progress(done, total, msg.to_string());
        if msg.starts_with("Packing") {
            client.push_log(msg);
        }
    });

    match res {
        Ok(()) => {
            controller.stop(true);
            Ok(SlopChopExit::Success)
        }
        Err(e) => {
            controller.stop(false);
            Err(e)
        }
    }
}

/// Handles the map command.
pub fn handle_map(deps: bool) -> Result<SlopChopExit> {
    let output = map::generate(deps)?;
    println!("{output}");
    Ok(SlopChopExit::Success)
}

/// Handles the signatures command.
pub fn handle_signatures(opts: SignatureOptions) -> Result<SlopChopExit> {
    signatures::run(&opts)?;
    Ok(SlopChopExit::Success)
}