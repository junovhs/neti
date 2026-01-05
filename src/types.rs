// src/types.rs
use serde::Serialize;
use std::path::PathBuf;

/// A single violation detected during analysis.
#[derive(Debug, Clone, Serialize)]
pub struct Violation {
    pub row: usize,
    pub message: String,
    pub law: &'static str,
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
    /// Creates a simple violation without details.
    #[must_use]
    pub fn simple(row: usize, message: String, law: &'static str) -> Self {
        Self { row, message, law, details: None }
    }

    /// Creates a violation with prescriptive details.
    #[must_use]
    pub fn with_details(
        row: usize,
        message: String,
        law: &'static str,
        details: ViolationDetails,
    ) -> Self {
        Self { row, message, law, details: Some(details) }
    }
}

/// Analysis results for a single file.
#[derive(Debug, Clone, Serialize)]
pub struct FileReport {
    pub path: PathBuf,
    pub token_count: usize,
    pub complexity_score: usize,
    pub violations: Vec<Violation>,
}

impl FileReport {
    /// Returns true if no violations were found.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }

    /// Returns the number of violations.
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
    /// Returns true if any violations were found.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.total_violations > 0
    }

    /// Returns the number of clean files.
    #[must_use]
    pub fn clean_file_count(&self) -> usize {
        self.files.iter().filter(|f| f.is_clean()).count()
    }
}

/// Result of an external command execution.
#[derive(Debug, Clone, Serialize)]
pub struct CommandResult {
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

/// Aggregated results for a full check run.
#[derive(Debug, Clone, Serialize)]
pub struct CheckReport {
    pub scan: ScanReport,
    pub commands: Vec<CommandResult>,
    pub passed: bool,
}
// test
