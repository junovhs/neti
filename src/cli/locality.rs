// src/cli/locality.rs
//! Handler for locality scanning.

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::discovery;
use crate::exit::NetiExit;
use crate::graph::locality::analysis::analyze;
use crate::graph::locality::coupling::compute_coupling;
use crate::graph::locality::report::print_full_report;
use crate::graph::locality::{collect_edges, validate_graph, Coupling};

/// Result of a locality check for use in verification pipeline.
pub struct LocalityResult {
    pub passed: bool,
    pub violations: usize,
}

/// Runs locality validation on the codebase.
///
/// # Errors
/// Returns error if file discovery or import extraction fails.
pub fn handle_locality() -> Result<NetiExit> {
    let result = run_locality_check(Path::new("."))?;
    
    if result.passed {
        Ok(NetiExit::Success)
    } else {
        Ok(NetiExit::CheckFailed)
    }
}

/// Runs locality check and returns result. Used by verification pipeline.
///
/// # Errors
/// Returns error if file discovery or graph construction fails.
pub fn run_locality_check(cwd: &Path) -> Result<LocalityResult> {
    let config = Config::load();
    
    if !config.rules.locality.is_enabled() {
        return Ok(LocalityResult { passed: true, violations: 0 });
    }

    let locality_config = config.rules.locality.to_validator_config();
    let project_root = if cwd == Path::new(".") {
        std::env::current_dir()?
    } else {
        cwd.to_path_buf()
    };

    let files = discovery::discover(&config)?;
    let edges = collect_edges(&project_root, &files)?;

    let couplings: HashMap<PathBuf, Coupling> = compute_coupling(
        edges.iter().map(|(a, b)| (a.as_path(), b.as_path())),
    );

    let report = validate_graph(
        edges.iter().map(|(a, b)| (a.as_path(), b.as_path())),
        &locality_config,
    );

    let violations = report.failed().len();
    let is_clean = report.is_clean();

    let analysis = analyze(&report, &couplings);
    print_full_report(&report, &analysis);

    let passed = is_clean || !config.rules.locality.is_error_mode();
    
    Ok(LocalityResult { passed, violations })
}

/// Runs locality check silently, returning only pass/fail. For pipeline use.
///
/// # Errors
/// Returns error if file discovery or graph construction fails.
pub fn check_locality_silent(cwd: &Path) -> Result<(bool, usize)> {
    let config = Config::load();
    
    if !config.rules.locality.is_enabled() {
        return Ok((true, 0));
    }

    let locality_config = config.rules.locality.to_validator_config();
    let project_root = if cwd == Path::new(".") {
        std::env::current_dir()?
    } else {
        cwd.to_path_buf()
    };

    let files = discovery::discover(&config)?;
    let edges = collect_edges(&project_root, &files)?;

    let report = validate_graph(
        edges.iter().map(|(a, b)| (a.as_path(), b.as_path())),
        &locality_config,
    );

    let violations = report.failed().len();
    let passed = report.is_clean() || !config.rules.locality.is_error_mode();
    
    Ok((passed, violations))
}

/// Returns whether locality is in error mode (blocking).
#[must_use]
pub fn is_locality_blocking() -> bool {
    Config::load().rules.locality.is_error_mode()
}