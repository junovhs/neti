// src/graph/locality/mod.rs
//! Law of Locality enforcement for topological integrity.
//!
//! Implements the Universal Locality Algorithm with smart exemptions
//! and deep analysis for actionable insights.

pub mod analysis;
pub mod classifier;
pub mod coupling;
pub mod cycles;
pub mod distance;
pub mod edges;
pub mod exemptions;
pub mod layers;
pub mod report;
pub mod types;
pub mod validator;

pub use classifier::{classify, ClassifierConfig};
pub use coupling::compute_coupling;
pub use distance::compute_distance;
pub use edges::collect as collect_edges;
pub use exemptions::is_structural_pattern;
pub use types::{Coupling, EdgeVerdict, LocalityEdge, NodeIdentity, PassReason};
pub use validator::{validate_edge, validate_graph, ValidationReport, ValidatorConfig};

#[cfg(test)]
mod tests;
