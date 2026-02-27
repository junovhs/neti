//! Types for locality (Law of Locality) reporting.

use serde::Serialize;
use std::path::PathBuf;

/// A single locality violation (edge that failed validation).
#[derive(Debug, Clone, Serialize)]
pub struct LocalityViolation {
    /// Source file of the dependency.
    pub from: PathBuf,
    /// Target file of the dependency.
    pub to: PathBuf,
    /// Directory distance between source and target.
    pub distance: usize,
    /// Classification of the target node.
    pub target_role: String,
}

/// Result of locality (Law of Locality) validation.
#[derive(Debug, Clone, Serialize)]
pub struct LocalityReport {
    /// Number of edge violations.
    pub violation_count: usize,
    /// Individual violation details.
    pub violations: Vec<LocalityViolation>,
    /// Number of dependency cycles detected.
    pub cycle_count: usize,
    /// Cycle paths (each cycle is a list of files).
    pub cycles: Vec<Vec<PathBuf>>,
    /// Total edges analyzed.
    pub total_edges: usize,
    /// Enforcement mode from config: "off", "warn", or "error".
    pub mode: String,
    /// Whether locality passed (clean, or mode is "warn").
    pub passed: bool,
}
