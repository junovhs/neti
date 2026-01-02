use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::graph::locality::types::{Coupling, LocalityEdge, NodeIdentity};



/// Categories of locality violations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ViolationKind {
    /// Importing module internals instead of public API.
    EncapsulationBreach,
    /// File depends on too many distant modules.
    GodModule,
    /// Target has high fan-in but isn't recognized as Hub.
    MissingHub,
    /// Generic sideways dependency.
    SidewaysDep,
    /// Dependency flows upwards (Lower Layer -> Higher Layer).
    UpwardDep,
}

impl ViolationKind {
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::EncapsulationBreach => "ENCAPSULATION_BREACH",
            Self::GodModule => "GOD_MODULE",
            Self::MissingHub => "MISSING_HUB",
            Self::SidewaysDep => "SIDEWAYS_DEP",
            Self::UpwardDep => "UPWARD_DEP",
        }
    }

    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::EncapsulationBreach => "Importing internal file instead of module API",
            Self::GodModule => "File has too many cross-boundary dependencies",
            Self::MissingHub => "Frequently imported file should be a Hub",
            Self::SidewaysDep => "Cross-module dependency without Hub routing",
            Self::UpwardDep => "Dependency violates architectural layering (Upward)",
        }
    }

    #[must_use]
    pub fn suggest(&self, edge: &LocalityEdge, fan_in: usize) -> String {
        match self {
            Self::EncapsulationBreach => suggest_encapsulation(edge),
            Self::GodModule => suggest_god_module(edge),
            Self::MissingHub => suggest_missing_hub(edge, fan_in),
            Self::SidewaysDep => suggest_sideways(edge),
            Self::UpwardDep => suggest_upward(edge),
        }
    }
}

/// A categorized violation with actionable suggestion.
#[derive(Debug, Clone)]
pub struct CategorizedViolation {
    pub edge: LocalityEdge,
    pub kind: ViolationKind,
    pub suggestion: String,
}

#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn categorize_violation(
    edge: &LocalityEdge,
    couplings: &HashMap<PathBuf, Coupling>,
    layers: &HashMap<PathBuf, usize>,
) -> ViolationKind {
    if is_internal_import(&edge.to) {
        return ViolationKind::EncapsulationBreach;
    }
    if is_missing_hub(edge, couplings) {
        return ViolationKind::MissingHub;
    }
    
    // Check for upward dependency
    let from_layer = layers.get(&edge.from).copied().unwrap_or(usize::MAX);
    let to_layer = layers.get(&edge.to).copied().unwrap_or(usize::MAX);
    if from_layer < to_layer {
        return ViolationKind::UpwardDep;
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

fn suggest_sideways(edge: &LocalityEdge) -> String {
    format!("Route through Hub or move '{}' closer", edge.to.display())
}

fn suggest_upward(edge: &LocalityEdge) -> String {
    format!("Layer violation. Move '{}' down to a lower layer or extract shared code.", edge.to.display())
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
