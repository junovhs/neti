//! Dead code detection using call graph reachability analysis.
//!
//! This module identifies code that is:
//! 1. Not reachable from any entry point (main, lib exports, tests)
//! 2. Defined but never referenced anywhere
//! 3. Only referenced by other dead code

pub mod analysis;
pub mod graph;

pub use analysis::extract_references;
use graph::CallGraph;

use crate::audit::types::{CodeUnit, DeadCode, DeadCodeReason};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// A node in the call graph.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Symbol {
    /// The defining file.
    pub file: PathBuf,
    /// The symbol name (fully qualified where possible).
    pub name: String,
}

/// Detects dead code from a list of code units and their references.
#[must_use]
pub fn detect(
    units: &[CodeUnit],
    references: &[(PathBuf, String, String)],
    entry_points: &[String],
) -> Vec<DeadCode> {
    let mut graph = CallGraph::new();

    for unit in units {
        let symbol = Symbol {
            file: unit.file.clone(),
            name: unit.name.clone(),
        };

        let is_entry = is_entry_point(&unit.name, &unit.file, entry_points);
        let is_public = is_likely_public(&unit.name);

        graph.add_symbol(symbol, is_public, is_entry);
    }

    for (file, from_name, to_name) in references {
        let from = Symbol {
            file: file.clone(),
            name: from_name.clone(),
        };
        let to = Symbol {
            file: file.clone(),
            name: to_name.clone(),
        };

        graph.add_edge(from, to);
    }

    let unreachable = find_unreachable(&graph);
    map_to_dead_code(units, unreachable)
}

fn find_unreachable(graph: &CallGraph) -> Vec<(Symbol, DeadCodeReason)> {
    let reachable = graph.compute_reachable();

    graph
        .symbols
        .iter()
        .filter(|sym| !reachable.contains(*sym))
        .map(|sym| {
            let reason = classify_dead_reason(sym, &graph.called_by, &reachable);
            (sym.clone(), reason)
        })
        .collect()
}

fn classify_dead_reason(
    sym: &Symbol,
    called_by: &HashMap<Symbol, HashSet<Symbol>>,
    reachable: &HashSet<Symbol>,
) -> DeadCodeReason {
    let Some(callers) = called_by.get(sym) else {
        return DeadCodeReason::Unused;
    };

    if callers.is_empty() {
        return DeadCodeReason::Unused;
    }

    let has_live_caller = callers.iter().any(|c| reachable.contains(c));
    if has_live_caller {
        DeadCodeReason::Unreachable
    } else {
        DeadCodeReason::OnlyDeadCallers
    }
}

fn map_to_dead_code(
    units: &[CodeUnit],
    unreachable: Vec<(Symbol, DeadCodeReason)>,
) -> Vec<DeadCode> {
    let unit_map: HashMap<(&PathBuf, &str), &CodeUnit> = units
        .iter()
        .map(|u| ((&u.file, u.name.as_str()), u))
        .collect();

    unreachable
        .into_iter()
        .filter_map(|(sym, reason)| {
            let unit = unit_map.get(&(&sym.file, sym.name.as_str()))?;
            Some(DeadCode {
                unit: (*unit).clone(),
                reason,
            })
        })
        .collect()
}

fn is_entry_point(name: &str, file: &Path, explicit_entries: &[String]) -> bool {
    if explicit_entries.contains(&name.to_string()) {
        return true;
    }

    if name == "main" {
        return true;
    }

    if name.starts_with("test_") || name.contains("_test") {
        return true;
    }

    let path_str = file.to_string_lossy();
    if path_str.contains("/bin/") || path_str.contains("/examples/") {
        return true;
    }

    false
}

fn is_likely_public(name: &str) -> bool {
    !name.starts_with('_')
}
