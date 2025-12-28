// src/cli/locality.rs
//! Handler for locality scanning.

use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::Config;
use crate::discovery;
use crate::exit::SlopChopExit;
use crate::graph::locality::analysis::analyze;
use crate::graph::locality::coupling::compute_coupling;
use crate::graph::locality::report::print_full_report;
use crate::graph::locality::{collect_edges, validate_graph, Coupling};

/// Runs locality validation on the codebase.
///
/// # Errors
/// Returns error if file discovery or import extraction fails.
pub fn handle_locality() -> Result<SlopChopExit> {
    let config = Config::load();
    let locality_config = config.rules.locality.to_validator_config();

    if !config.rules.locality.is_enabled() {
        println!("{}", "Locality checking is disabled.".yellow());
        return Ok(SlopChopExit::Success);
    }

    let project_root = std::env::current_dir()?;
    let files = discovery::discover(&config)?;
    let edges = collect_edges(&project_root, &files)?;

    let couplings: HashMap<PathBuf, Coupling> = compute_coupling(
        edges.iter().map(|(a, b)| (a.as_path(), b.as_path())),
    );

    let report = validate_graph(
        edges.iter().map(|(a, b)| (a.as_path(), b.as_path())),
        &locality_config,
    );

    let analysis = analyze(&report, &couplings);
    print_full_report(&report, &analysis);

    if report.is_clean() || !config.rules.locality.is_error_mode() {
        Ok(SlopChopExit::Success)
    } else {
        Ok(SlopChopExit::CheckFailed)
    }
}