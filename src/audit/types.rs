// src/audit/types.rs
//! Core types for the consolidation audit system.
//!
//! This module defines the data structures used to represent code duplication,
//! dead code, and consolidation opportunities.

use std::collections::HashSet;
use std::path::PathBuf;

/// A structural fingerprint of a code unit (function, struct, impl block).
/// Uses a hash that is invariant to identifier names but sensitive to structure.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Fingerprint {
    /// The structural hash (Weisfeiler-Lehman style).
    pub hash: u64,
    /// Depth of the AST subtree.
    pub depth: usize,
    /// Number of nodes in the subtree.
    pub node_count: usize,
}

/// A code unit that can be fingerprinted and compared.
#[derive(Debug, Clone)]
pub struct CodeUnit {
    /// File containing this unit.
    pub file: PathBuf,
    /// Name of the unit (function name, struct name, etc.).
    pub name: String,
    /// The kind of code unit.
    pub kind: CodeUnitKind,
    /// Line number where this unit starts.
    pub start_line: usize,
    /// Line number where this unit ends.
    pub end_line: usize,
    /// The structural fingerprint.
    pub fingerprint: Fingerprint,
    /// Estimated token count.
    pub tokens: usize,
}

impl CodeUnit {
    /// Returns the number of lines in this code unit.
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.end_line.saturating_sub(self.start_line) + 1
    }
}

/// The kind of code unit being analyzed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CodeUnitKind {
    Function,
    Method,
    Struct,
    Enum,
    Trait,
    Impl,
    Module,
}

impl CodeUnitKind {
    /// Returns a human-readable label.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Method => "method",
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Trait => "trait",
            Self::Impl => "impl",
            Self::Module => "module",
        }
    }
}

/// A cluster of similar code units (potential duplicates).
#[derive(Debug, Clone)]
pub struct SimilarityCluster {
    /// The units in this cluster.
    pub units: Vec<CodeUnit>,
    /// Similarity score (0.0 to 1.0).
    pub similarity: f64,
    /// Estimated lines that could be saved by consolidation.
    pub potential_savings: usize,
}

impl SimilarityCluster {
    /// Returns the number of duplicates (units - 1).
    #[must_use]
    pub fn duplicate_count(&self) -> usize {
        self.units.len().saturating_sub(1)
    }
}

/// A code unit that appears to be unreachable (dead code).
#[derive(Debug, Clone)]
pub struct DeadCode {
    /// The unreachable unit.
    pub unit: CodeUnit,
    /// Why we believe it's dead.
    pub reason: DeadCodeReason,
}

/// Reason a code unit is considered dead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeadCodeReason {
    /// Not reachable from any entry point.
    Unreachable,
    /// Defined but never referenced.
    Unused,
    /// Only referenced by other dead code.
    OnlyDeadCallers,
}

impl DeadCodeReason {
    /// Returns a human-readable explanation.
    #[must_use]
    pub fn explanation(&self) -> &'static str {
        match self {
            Self::Unreachable => "not reachable from any entry point",
            Self::Unused => "defined but never referenced",
            Self::OnlyDeadCallers => "only called by other dead code",
        }
    }
}

/// A detected pattern that appears multiple times.
#[derive(Debug, Clone)]
pub struct RepeatedPattern {
    /// Human-readable description of the pattern.
    pub description: String,
    /// Locations where this pattern appears.
    pub locations: Vec<PatternLocation>,
    /// The canonical pattern signature.
    pub signature: String,
    /// Estimated savings from extracting this pattern.
    pub potential_savings: usize,
}

/// Location of a pattern occurrence.
#[derive(Debug, Clone)]
pub struct PatternLocation {
    pub file: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
}

/// A consolidation opportunity with impact scoring.
#[derive(Debug, Clone)]
pub struct Opportunity {
    /// Unique identifier for this opportunity.
    pub id: String,
    /// Human-readable title.
    pub title: String,
    /// Detailed description.
    pub description: String,
    /// The kind of opportunity.
    pub kind: OpportunityKind,
    /// Impact score (higher = more impactful).
    pub impact: Impact,
    /// Files affected by this opportunity.
    pub affected_files: HashSet<PathBuf>,
    /// Specific recommendation.
    pub recommendation: String,
}

/// The kind of consolidation opportunity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpportunityKind {
    /// Near-duplicate code that could be merged.
    Duplication,
    /// Dead code that could be removed.
    DeadCode,
    /// Repeated pattern that could be extracted.
    Pattern,
    /// Module that could be simplified or merged.
    ModuleConsolidation,
}

impl OpportunityKind {
    /// Returns a severity label for display.
    #[must_use]
    pub fn severity(&self) -> &'static str {
        match self {
            Self::DeadCode => "LOW",
            Self::Pattern => "MEDIUM",
            Self::Duplication => "HIGH",
            Self::ModuleConsolidation => "HIGH",
        }
    }
}

/// Impact score for prioritization.
#[derive(Debug, Clone, Copy)]
pub struct Impact {
    /// Estimated lines that could be removed/consolidated.
    pub lines_saved: usize,
    /// Estimated tokens saved.
    pub tokens_saved: usize,
    /// Difficulty of the refactor (1 = easy, 5 = hard).
    pub difficulty: u8,
    /// Confidence in this estimate (0.0 to 1.0).
    pub confidence: f64,
}

impl Impact {
    /// Computes a composite score for sorting.
    /// Higher is better (more impact, easier to do).
    #[must_use]
    pub fn score(&self) -> f64 {
        let base = self.lines_saved as f64;
        let difficulty_factor = 1.0 / (self.difficulty as f64).max(1.0);
        base * difficulty_factor * self.confidence
    }
}

/// The complete audit report.
#[derive(Debug, Clone, Default)]
pub struct AuditReport {
    /// All detected opportunities, sorted by impact.
    pub opportunities: Vec<Opportunity>,
    /// Summary statistics.
    pub stats: AuditStats,
}

/// Summary statistics from the audit.
#[derive(Debug, Clone, Default)]
pub struct AuditStats {
    /// Total files analyzed.
    pub files_analyzed: usize,
    /// Total code units extracted.
    pub units_extracted: usize,
    /// Total similarity clusters found.
    pub similarity_clusters: usize,
    /// Total dead code units found.
    pub dead_code_units: usize,
    /// Total repeated patterns found.
    pub pattern_instances: usize,
    /// Estimated total lines savable.
    pub total_potential_savings: usize,
    /// Analysis duration in milliseconds.
    pub duration_ms: u128,
}

