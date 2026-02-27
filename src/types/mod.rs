use serde::Serialize;
use std::path::PathBuf;

use crate::analysis::aggregator::FileAnalysis;

mod command;
mod locality;
pub use command::CommandResult;
pub use locality::{LocalityReport, LocalityViolation};

/// Confidence level for a violation — how certain Neti is that this is a real problem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum Confidence {
    /// Style observation. Not wrong, but could be improved.
    Info,
    /// Neti sees a suspicious pattern but cannot prove it's wrong.
    /// May require type information, algorithmic intent, or cross-scope
    /// analysis that tree-sitter cannot provide.
    Medium,
    /// Neti can prove this is wrong. Structural violation, missing required
    /// annotation, provable bounds error.
    High,
}

impl Confidence {
    /// Label shown in the report output.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::High => "Fix required",
            Self::Medium => "Review recommended",
            Self::Info => "Style suggestion",
        }
    }

    /// Prefix word for the report line (error/warn/info).
    #[must_use]
    pub fn prefix(self) -> &'static str {
        match self {
            Self::High => "error",
            Self::Medium => "warn",
            Self::Info => "info",
        }
    }
}

/// A single violation detected during analysis.
#[derive(Debug, Clone, Serialize)]
pub struct Violation {
    pub row: usize,
    pub message: String,
    pub law: &'static str,
    pub confidence: Confidence,
    /// Why Neti can't be fully certain (for Medium confidence).
    /// Shown after "Review recommended — {reason}".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence_reason: Option<String>,
    pub details: Option<ViolationDetails>,
}

/// Rich details for prescriptive violation reporting.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ViolationDetails {
    pub function_name: Option<String>,
    pub analysis: Vec<String>,
    pub suggestion: Option<String>,
}

impl Violation {
    #[must_use]
    pub fn simple(row: usize, message: String, law: &'static str) -> Self {
        Self {
            row,
            message,
            law,
            confidence: Confidence::High,
            confidence_reason: None,
            details: None,
        }
    }

    #[must_use]
    pub fn with_details(
        row: usize,
        message: String,
        law: &'static str,
        details: ViolationDetails,
    ) -> Self {
        Self {
            row,
            message,
            law,
            confidence: Confidence::High,
            confidence_reason: None,
            details: Some(details),
        }
    }
}

/// Analysis results for a single file.
#[derive(Debug, Clone, Serialize)]
pub struct FileReport {
    pub path: PathBuf,
    pub token_count: usize,
    pub complexity_score: usize,
    pub violations: Vec<Violation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analysis: Option<FileAnalysis>,
}

impl FileReport {
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }

    #[must_use]
    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }
}

/// Aggregated results from scanning multiple files.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ScanReport {
    pub files: Vec<FileReport>,
    pub total_tokens: usize,
    pub total_violations: usize,
    pub duration_ms: u128,
}

impl ScanReport {
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.total_violations > 0
    }

    /// Count of violations at HIGH confidence (proven errors).
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.files
            .iter()
            .flat_map(|f| &f.violations)
            .filter(|v| v.confidence == Confidence::High)
            .count()
    }

    /// Count of violations at MEDIUM confidence (review recommended).
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.files
            .iter()
            .flat_map(|f| &f.violations)
            .filter(|v| v.confidence == Confidence::Medium)
            .count()
    }

    /// Count of violations at INFO confidence (style suggestions).
    #[must_use]
    pub fn suggestion_count(&self) -> usize {
        self.files
            .iter()
            .flat_map(|f| &f.violations)
            .filter(|v| v.confidence == Confidence::Info)
            .count()
    }

    /// Returns `true` if any HIGH confidence violations exist.
    #[must_use]
    pub fn has_blocking_errors(&self) -> bool {
        self.error_count() > 0
    }

    #[must_use]
    pub fn clean_file_count(&self) -> usize {
        self.files.iter().filter(|f| f.is_clean()).count()
    }

    #[must_use]
    pub fn is_small_codebase(&self) -> bool {
        crate::analysis::Engine::small_codebase_threshold() >= self.files.len()
    }
}

/// Aggregated results for a full check run.
#[derive(Debug, Clone, Serialize)]
pub struct CheckReport {
    pub scan: ScanReport,
    pub commands: Vec<CommandResult>,
    /// Locality analysis results, if enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locality: Option<LocalityReport>,
    pub passed: bool,
}
