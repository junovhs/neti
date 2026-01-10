use std::collections::HashMap;
use std::path::PathBuf;

use crate::graph::locality::types::{Coupling, LocalityEdge};
use crate::graph::locality::ValidationReport;

use super::violations::CategorizedViolation;

#[derive(Debug, Clone)]
pub struct GodModuleInfo {
    pub path: PathBuf,
    pub outbound_violations: usize,
    pub targets: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct HubCandidate {
    pub path: PathBuf,
    pub fan_in: usize,
    pub importers: Vec<PathBuf>,
}

#[must_use]
pub fn find_god_modules(violations: &[CategorizedViolation]) -> Vec<GodModuleInfo> {
    let mut by_source: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

    for v in violations {
        by_source
            .entry(v.edge.from.clone())
            .or_default()
            .push(v.edge.to.clone());
    }

    by_source
        .into_iter()
        .filter(|(_, targets)| targets.len() >= 3)
        .map(|(path, targets)| GodModuleInfo {
            path,
            outbound_violations: targets.len(),
            targets,
        })
        .collect()
}

#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn find_hub_candidates(
    couplings: &HashMap<PathBuf, Coupling>,
    report: &ValidationReport,
) -> Vec<HubCandidate> {
    let mut target_importers: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

    for edge in report.failed() {
        target_importers
            .entry(edge.to.clone())
            .or_default()
            .push(edge.from.clone());
    }

    target_importers
        .into_iter()
        .filter(|(_, importers)| importers.len() >= 2)
        .map(|(path, importers)| {
            let fan_in = couplings.get(&path).map_or(0, Coupling::afferent);
            HubCandidate { path, fan_in, importers }
        })
        .collect()
}

#[must_use]
pub fn compute_module_coupling(edges: &[LocalityEdge]) -> Vec<(String, String, usize)> {
    let mut coupling: HashMap<(String, String), usize> = HashMap::new();

    for edge in edges {
        let from_mod = get_top_module(&edge.from);
        let to_mod = get_top_module(&edge.to);
        if from_mod != to_mod {
            let key = order_pair(from_mod, to_mod);
            *coupling.entry(key).or_insert(0) += 1;
        }
    }

    let mut result: Vec<_> = coupling.into_iter().map(|((a, b), c)| (a, b, c)).collect();
    result.sort_by(|a, b| b.2.cmp(&a.2));
    result
}

fn get_top_module(path: &std::path::Path) -> String {
    let parts: Vec<_> = path.components().collect();
    let src_idx = parts.iter().position(|c| c.as_os_str() == "src");
    src_idx
        .and_then(|i| parts.get(i + 1))
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("unknown")
        .to_string()
}

fn order_pair(a: String, b: String) -> (String, String) {
    if a < b { (a, b) } else { (b, a) }
}