// src/graph/locality/analysis.rs
//! Deep topology analysis: categorize violations, find patterns, suggest fixes.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::types::{Coupling, LocalityEdge, NodeIdentity};
use super::ValidationReport;

/// Categories of locality violations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ViolationKind {
    /// Importing module internals instead of public API.
    EncapsulationBreach,
    /// File depends on too many distant modules.
    GodModule,
    /// Target has high fan-in but isn't recognized as Hub.
    MissingHub,
    /// Two subsystems are too tightly coupled.
    TightCoupling,
    /// Generic sideways dependency.
    SidewaysDep,
}

impl ViolationKind {
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::EncapsulationBreach => "ENCAPSULATION_BREACH",
            Self::GodModule => "GOD_MODULE",
            Self::MissingHub => "MISSING_HUB",
            Self::TightCoupling => "TIGHT_COUPLING",
            Self::SidewaysDep => "SIDEWAYS_DEP",
        }
    }

    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::EncapsulationBreach => "Importing internal file instead of module API",
            Self::GodModule => "File has too many cross-boundary dependencies",
            Self::MissingHub => "Frequently imported file should be a Hub",
            Self::TightCoupling => "Subsystems are too intertwined",
            Self::SidewaysDep => "Cross-module dependency without Hub routing",
        }
    }

    #[must_use]
    pub fn suggest(&self, edge: &LocalityEdge, fan_in: usize) -> String {
        match self {
            Self::EncapsulationBreach => suggest_encapsulation(edge),
            Self::GodModule => suggest_god_module(edge),
            Self::MissingHub => suggest_missing_hub(edge, fan_in),
            Self::TightCoupling => suggest_tight_coupling(edge),
            Self::SidewaysDep => suggest_sideways(edge),
        }
    }
}

fn suggest_encapsulation(edge: &LocalityEdge) -> String {
    let module = get_module_root(&edge.to);
    format!(
        "Expose needed API from '{}' instead of importing '{}'",
        module.display(),
        edge.to.display()
    )
}

fn suggest_god_module(edge: &LocalityEdge) -> String {
    format!("Split '{}' into focused handlers", edge.from.display())
}

fn suggest_missing_hub(edge: &LocalityEdge, fan_in: usize) -> String {
    format!(
        "Promote '{}' to Hub (fan-in: {fan_in}). Add to [rules.locality].hubs",
        edge.to.display()
    )
}

fn suggest_tight_coupling(edge: &LocalityEdge) -> String {
    let from_mod = get_top_module(&edge.from);
    let to_mod = get_top_module(&edge.to);
    format!("'{from_mod}' ↔ '{to_mod}' coupled. Merge or extract shared interface")
}

fn suggest_sideways(edge: &LocalityEdge) -> String {
    format!("Route through Hub or move '{}' closer", edge.to.display())
}

/// A categorized violation with actionable suggestion.
#[derive(Debug, Clone)]
pub struct CategorizedViolation {
    pub edge: LocalityEdge,
    pub kind: ViolationKind,
    pub suggestion: String,
}

/// Analysis results with insights.
#[derive(Debug, Default)]
pub struct TopologyAnalysis {
    pub violations: Vec<CategorizedViolation>,
    pub god_modules: Vec<GodModuleInfo>,
    pub hub_candidates: Vec<HubCandidate>,
    pub module_coupling: Vec<(String, String, usize)>,
}

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

/// Analyzes a validation report and produces actionable insights.
#[allow(clippy::implicit_hasher)]
#[must_use]
pub fn analyze(
    report: &ValidationReport,
    couplings: &HashMap<PathBuf, Coupling>,
) -> TopologyAnalysis {
    let mut analysis = TopologyAnalysis::default();

    for edge in &report.failed {
        let kind = categorize_violation(edge, couplings);
        let fan_in = couplings.get(&edge.to).map_or(0, |c| c.afferent);
        let suggestion = kind.suggest(edge, fan_in);
        analysis.violations.push(CategorizedViolation {
            edge: edge.clone(),
            kind,
            suggestion,
        });
    }

    analysis.god_modules = find_god_modules(&analysis.violations);
    analysis.hub_candidates = find_hub_candidates(couplings, report);
    analysis.module_coupling = compute_module_coupling(&report.failed);
    analysis
}

fn categorize_violation(
    edge: &LocalityEdge,
    couplings: &HashMap<PathBuf, Coupling>,
) -> ViolationKind {
    if is_internal_import(&edge.to) {
        return ViolationKind::EncapsulationBreach;
    }
    if is_missing_hub(edge, couplings) {
        return ViolationKind::MissingHub;
    }
    if edge.target_identity == NodeIdentity::VolatileLeaf {
        return ViolationKind::TightCoupling;
    }
    ViolationKind::SidewaysDep
}

fn is_missing_hub(edge: &LocalityEdge, couplings: &HashMap<PathBuf, Coupling>) -> bool {
    couplings.get(&edge.to).is_some_and(|c| {
        c.afferent >= 3 && edge.target_identity != NodeIdentity::StableHub
    })
}

fn is_internal_import(path: &Path) -> bool {
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    name != "mod.rs" && path.components().count() > 2
}

fn get_module_root(path: &Path) -> PathBuf {
    let mut parts: Vec<_> = path.components().collect();
    if let Some(last) = parts.last() {
        if last.as_os_str() != "mod.rs" {
            parts.pop();
            parts.push(std::path::Component::Normal("mod.rs".as_ref()));
        }
    }
    parts.iter().collect()
}

fn get_top_module(path: &Path) -> String {
    let parts: Vec<_> = path.components().collect();
    let src_idx = parts.iter().position(|c| c.as_os_str() == "src");
    src_idx
        .and_then(|i| parts.get(i + 1))
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("unknown")
        .to_string()
}

fn find_god_modules(violations: &[CategorizedViolation]) -> Vec<GodModuleInfo> {
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

fn find_hub_candidates(
    couplings: &HashMap<PathBuf, Coupling>,
    report: &ValidationReport,
) -> Vec<HubCandidate> {
    let mut target_importers: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

    for edge in &report.failed {
        target_importers
            .entry(edge.to.clone())
            .or_default()
            .push(edge.from.clone());
    }

    target_importers
        .into_iter()
        .filter(|(_, importers)| importers.len() >= 2)
        .map(|(path, importers)| {
            let fan_in = couplings.get(&path).map_or(0, |c| c.afferent);
            HubCandidate { path, fan_in, importers }
        })
        .collect()
}

fn compute_module_coupling(edges: &[LocalityEdge]) -> Vec<(String, String, usize)> {
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

fn order_pair(a: String, b: String) -> (String, String) {
    if a < b { (a, b) } else { (b, a) }
}