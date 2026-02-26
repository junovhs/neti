// src/graph/locality/types.rs
//! Core types for the Law of Locality enforcement system.

use std::path::PathBuf;

/// Coupling metrics for a single file node.
#[derive(Debug, Clone, Default)]
pub struct Coupling {
    /// Afferent coupling (fan-in): files that depend ON this file.
    afferent: usize,
    /// Efferent coupling (fan-out): files this file depends ON.
    efferent: usize,
}

impl Coupling {
    #[must_use]
    pub fn new(afferent: usize, efferent: usize) -> Self {
        Self { afferent, efferent }
    }

    /// Instability Index: I = Cₑ / (Cₐ + Cₑ).
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn instability(&self) -> f64 {
        let total = self.afferent + self.efferent;
        if total == 0 {
            return 0.0;
        }
        self.efferent as f64 / total as f64
    }

    /// Skew Score: K = ln((Cₐ + 1) / (Cₑ + 1)).
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn skew(&self) -> f64 {
        let numerator = (self.afferent + 1) as f64;
        let denominator = (self.efferent + 1) as f64;
        (numerator / denominator).ln()
    }

    #[must_use]
    pub fn total(&self) -> usize {
        self.afferent + self.efferent
    }

    #[must_use] pub fn afferent(&self) -> usize { self.afferent }
    #[must_use] pub fn efferent(&self) -> usize { self.efferent }
}

/// Classification of a node's role in the dependency topology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeIdentity {
    StableHub,
    VolatileLeaf,
    IsolatedDeadwood,
    GodModule,
    Standard,
}

impl NodeIdentity {
    #[must_use]
    pub fn allows_far_deps(&self) -> bool {
        matches!(self, Self::StableHub)
    }

    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::StableHub => "STABLE_HUB",
            Self::VolatileLeaf => "VOLATILE_LEAF",
            Self::IsolatedDeadwood => "DEADWOOD",
            Self::GodModule => "GOD_MODULE",
            Self::Standard => "STANDARD",
        }
    }
}

/// A dependency edge with computed locality metrics.
#[derive(Debug, Clone)]
pub struct LocalityEdge {
    pub from: PathBuf,
    pub to: PathBuf,
    pub distance: usize,
    pub target_skew: f64,
    pub target_identity: NodeIdentity,
}

impl LocalityEdge {
    #[must_use]
    pub fn is_local(&self, max_distance: usize) -> bool {
        self.distance <= max_distance
    }

    #[must_use]
    pub fn routes_to_hub(&self, hub_threshold: f64) -> bool {
        self.target_skew >= hub_threshold
    }
}

/// Result of validating a single edge.
#[derive(Debug, Clone)]
pub enum EdgeVerdict {
    Pass { reason: PassReason },
    Fail { edge: LocalityEdge, suggestion: String },
}

/// Reason an edge passed validation.
#[derive(Debug, Clone, Copy)]
pub enum PassReason {
    L1Cache,
    WithinDistance,
    VerticalRouting,
    Exempted,
}
