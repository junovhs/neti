//! Deep topology analysis: categorize violations, find patterns, suggest fixes.

pub mod metrics;
pub mod violations;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::graph::locality::types::Coupling;
use crate::graph::locality::ValidationReport;

pub use self::metrics::{GodModuleInfo, HubCandidate};
pub use self::violations::{CategorizedViolation, ViolationKind};

use self::metrics::{compute_module_coupling, find_god_modules, find_hub_candidates};
use self::violations::categorize_violation;

/// Analysis results with insights.
#[derive(Debug, Default)]
pub struct TopologyAnalysis {
    pub violations: Vec<CategorizedViolation>,
    pub god_modules: Vec<GodModuleInfo>,
    pub hub_candidates: Vec<HubCandidate>,
    pub module_coupling: Vec<(String, String, usize)>,
}

/// Analyzes a validation report and produces actionable insights.
#[allow(clippy::implicit_hasher)]
#[must_use]
pub fn analyze(
    report: &ValidationReport,
    couplings: &HashMap<PathBuf, Coupling>,
) -> TopologyAnalysis {
    let mut analysis = TopologyAnalysis::default();

    for edge in report.failed() {
        let kind = categorize_violation(edge, couplings, report.layers());
        let fan_in = couplings.get(&edge.to).map_or(0, Coupling::afferent);
        let suggestion = kind.suggest(edge, fan_in);
        analysis.violations.push(CategorizedViolation {
            edge: edge.clone(),
            kind,
            suggestion,
        });
    }

    analysis.god_modules = find_god_modules(&analysis.violations);
    analysis.hub_candidates = find_hub_candidates(couplings, report);
    analysis.module_coupling = compute_module_coupling(report.failed());
    analysis
}