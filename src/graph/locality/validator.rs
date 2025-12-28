// src/graph/locality/validator.rs
//! The Universal Locality Algorithm - Judgment Pass.

use std::path::{Path, PathBuf};

use super::classifier::{classify, ClassifierConfig};
use super::coupling::compute_coupling;
use super::distance::compute_distance;
use super::exemptions::is_structural_pattern;
use super::types::{Coupling, EdgeVerdict, LocalityEdge, NodeIdentity, PassReason};

/// Configuration for locality validation.
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    pub max_distance: usize,
    pub l1_threshold: usize,
    pub classifier: ClassifierConfig,
    pub manual_hubs: Vec<PathBuf>,
    pub exempt_patterns: Vec<String>,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            max_distance: 4,
            l1_threshold: 2,
            classifier: ClassifierConfig::default(),
            manual_hubs: Vec::new(),
            exempt_patterns: Vec::new(),
        }
    }
}

/// Result of validating all edges.
#[derive(Debug, Default)]
pub struct ValidationReport {
    pub passed: Vec<LocalityEdge>,
    pub failed: Vec<LocalityEdge>,
    pub total_edges: usize,
    pub entropy: f64,
}

impl ValidationReport {
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.failed.is_empty()
    }

    #[allow(clippy::cast_precision_loss)]
    fn compute_entropy(&mut self) {
        if self.total_edges == 0 {
            self.entropy = 0.0;
            return;
        }
        self.entropy = self.failed.len() as f64 / self.total_edges as f64;
    }
}

/// Validates a single edge against locality rules.
#[must_use]
pub fn validate_edge(
    from: &Path,
    to: &Path,
    target_coupling: &Coupling,
    config: &ValidatorConfig,
) -> EdgeVerdict {
    let edge = build_locality_edge(from, to, target_coupling, config);

    // Check structural patterns first (lib.rs, mod.rs re-exports, vertical)
    if is_structural_pattern(from, to) {
        return EdgeVerdict::Pass { reason: PassReason::Exempted };
    }

    if let Some(reason) = check_distance(&edge, config) {
        return EdgeVerdict::Pass { reason };
    }
    if let Some(reason) = check_hub_status(&edge, to, config) {
        return EdgeVerdict::Pass { reason };
    }

    let suggestion = generate_suggestion(&edge, target_coupling);
    EdgeVerdict::Fail { edge, suggestion }
}

fn build_locality_edge(
    from: &Path,
    to: &Path,
    coupling: &Coupling,
    config: &ValidatorConfig,
) -> LocalityEdge {
    LocalityEdge {
        from: from.to_path_buf(),
        to: to.to_path_buf(),
        distance: compute_distance(from, to),
        target_skew: coupling.skew(),
        target_identity: classify(coupling, &config.classifier),
    }
}

fn check_distance(edge: &LocalityEdge, config: &ValidatorConfig) -> Option<PassReason> {
    if edge.distance <= config.l1_threshold {
        return Some(PassReason::L1Cache);
    }
    if edge.distance <= config.max_distance {
        return Some(PassReason::WithinDistance);
    }
    None
}

fn check_hub_status(
    edge: &LocalityEdge,
    to: &Path,
    config: &ValidatorConfig,
) -> Option<PassReason> {
    if config.manual_hubs.contains(&to.to_path_buf()) {
        return Some(PassReason::VerticalRouting);
    }
    if edge.target_identity == NodeIdentity::StableHub {
        return Some(PassReason::VerticalRouting);
    }
    if is_exempt(to, &config.exempt_patterns) {
        return Some(PassReason::Exempted);
    }
    None
}

fn is_exempt(path: &Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    patterns.iter().any(|p| match_pattern(p, &path_str))
}

fn match_pattern(pattern: &str, path_str: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix("/*") {
        path_str.starts_with(prefix)
    } else {
        path_str == pattern
    }
}

fn generate_suggestion(edge: &LocalityEdge, coupling: &Coupling) -> String {
    if coupling.afferent > 3 {
        format!(
            "Target '{}' has high fan-in (Ca={}). Consider promoting to Hub.",
            edge.to.display(),
            coupling.afferent
        )
    } else {
        format!(
            "Sideways dep: {} → {}. Move closer or extract shared Hub.",
            edge.from.display(),
            edge.to.display()
        )
    }
}

/// Validates all edges in a dependency graph.
pub fn validate_graph<'a, I>(edges: I, config: &ValidatorConfig) -> ValidationReport
where
    I: Iterator<Item = (&'a Path, &'a Path)> + Clone,
{
    let coupling_map = compute_coupling(edges.clone());
    let mut report = ValidationReport::default();

    for (from, to) in edges {
        report.total_edges += 1;
        let target_coupling = coupling_map.get(to).cloned().unwrap_or_default();
        process_edge(from, to, &target_coupling, config, &mut report);
    }

    report.compute_entropy();
    report
}

fn process_edge(
    from: &Path,
    to: &Path,
    coupling: &Coupling,
    config: &ValidatorConfig,
    report: &mut ValidationReport,
) {
    match validate_edge(from, to, coupling, config) {
        EdgeVerdict::Pass { .. } => {
            let edge = build_locality_edge(from, to, coupling, config);
            report.passed.push(edge);
        }
        EdgeVerdict::Fail { edge, .. } => {
            report.failed.push(edge);
        }
    }
}