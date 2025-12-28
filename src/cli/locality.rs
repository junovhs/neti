// src/cli/locality.rs
//! Handler for locality scanning.

use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::discovery;
use crate::exit::SlopChopExit;
use crate::graph::imports;
use crate::graph::locality::analysis::analyze;
use crate::graph::locality::coupling::compute_coupling;
use crate::graph::locality::report::print_full_report;
use crate::graph::locality::{validate_graph, Coupling};
use crate::graph::resolver;

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

    // Compute couplings for analysis
    let couplings: HashMap<PathBuf, Coupling> = compute_coupling(
        edges.iter().map(|(a, b)| (a.as_path(), b.as_path())),
    );

    let report = validate_graph(
        edges.iter().map(|(a, b)| (a.as_path(), b.as_path())),
        &locality_config,
    );

    // Deep analysis
    let analysis = analyze(&report, &couplings);

    // Rich output
    print_full_report(&report, &analysis);

    if report.is_clean() || !config.rules.locality.is_error_mode() {
        Ok(SlopChopExit::Success)
    } else {
        Ok(SlopChopExit::CheckFailed)
    }
}

fn collect_edges(root: &Path, files: &[PathBuf]) -> Result<Vec<(PathBuf, PathBuf)>> {
    let mut edges = Vec::new();

    for file in files {
        let content = std::fs::read_to_string(file)?;
        let raw_imports = imports::extract(file, &content);

        for import_str in raw_imports {
            if let Some(resolved) = resolver::resolve(root, file, &import_str) {
                let from = normalize_path(file, root);
                let to = normalize_path(&resolved, root);
                edges.push((from, to));
            }
        }
    }

    Ok(edges)
}

/// Strips the project root to get a relative path for consistent D calculation.
fn normalize_path(path: &Path, root: &Path) -> PathBuf {
    path.strip_prefix(root)
        .map_or_else(|_| path.to_path_buf(), Path::to_path_buf)
}