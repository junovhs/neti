// src/audit/types.rs
//! Core types for the consolidation audit system.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// A structural fingerprint of a code unit (function, struct, impl block).
/// Uses multiple hash strategies for semantic similarity detection.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Fingerprint {
    /// Full structural hash (Weisfeiler-Lehman style, identifier-invariant).
    pub hash: u64,
    /// Control flow only hash - ignores expressions, captures branching structure.
    pub cfg_hash: u64,
    /// Depth of the AST subtree.
    pub depth: usize,
    /// Number of nodes in the subtree.
    pub node_count: usize,
    /// Number of branch points (if, match arms, ?).
    pub branch_count: usize,
    /// Number of loop constructs (for, while, loop).
    pub loop_count: usize,
    /// Number of return/break/continue points.
    pub exit_count: usize,
}

/// A code unit that can be fingerprinted and compared.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeUnit {
    pub file: PathBuf,
    pub name: String,
    pub kind: CodeUnitKind,
    pub start_line: usize,
    pub end_line: usize,
    pub fingerprint: Fingerprint,
    pub tokens: usize,
    /// Semantic signature tokens (e.g., enum variants).
    #[serde(default)]
    pub signature: Vec<String>,
}

impl CodeUnit {
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.end_line.saturating_sub(self.start_line) + 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    #[must_use]
    pub const fn label(self) -> &'static str {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityCluster {
    pub units: Vec<CodeUnit>,
    pub similarity: f64,
    pub potential_savings: usize,
}

impl SimilarityCluster {
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.units.iter().map(|u| u.name.as_str()).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCode {
    pub unit: CodeUnit,
    pub reason: DeadCodeReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeadCodeReason {
    Unreachable,
    Unused,
    OnlyDeadCallers,
}

impl DeadCodeReason {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unreachable => "unreachable",
            Self::Unused => "unused",
            Self::OnlyDeadCallers => "only dead callers",
        }
    }

    #[must_use]
    pub const fn explanation(self) -> &'static str {
        match self {
            Self::Unreachable => "unreachable from entry points",
            Self::Unused => "defined but never used",
            Self::OnlyDeadCallers => "only called by dead code",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatedPattern {
    pub description: String,
    pub locations: Vec<PatternLocation>,
    pub signature: String,
    pub potential_savings: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternLocation {
    pub file: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opportunity {
    pub id: String,
    pub title: String,
    pub description: String,
    pub kind: OpportunityKind,
    pub impact: Impact,
    pub affected_files: HashSet<PathBuf>,
    pub recommendation: String,
    pub refactoring_plan: Option<String>,
    #[serde(skip)]
    pub units: Vec<CodeUnit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OpportunityKind {
    Duplication,
    DeadCode,
    Pattern,
    ModuleConsolidation,
}

impl OpportunityKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Duplication => "duplication",
            Self::DeadCode => "dead code",
            Self::Pattern => "pattern",
            Self::ModuleConsolidation => "module consolidation",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Impact {
    pub lines_saved: usize,
    pub tokens_saved: usize,
    pub difficulty: u8,
    pub confidence: f64,
}

impl Impact {
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn score(&self) -> f64 {
        let base = self.lines_saved as f64;
        let diff_penalty = 1.0 / f64::from(self.difficulty).max(1.0);
        base * self.confidence * diff_penalty
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditReport {
    pub opportunities: Vec<Opportunity>,
    pub stats: AuditStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditStats {
    pub files_analyzed: usize,
    pub units_extracted: usize,
    pub similarity_clusters: usize,
    pub dead_code_units: usize,
    pub pattern_instances: usize,
    pub total_potential_savings: usize,
    pub duration_ms: u128,
}

/// Represents a call site for dead code analysis.
#[derive(Debug, Clone)]
pub struct CallSite {
    pub file: PathBuf,
    pub caller: String,
    pub callee: String,
    pub line: usize,
}