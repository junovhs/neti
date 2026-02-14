//! Types for the apply module.

use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::analysis::aggregator::FileAnalysis;

// =============================================================================
// Shared Types (duplicated from src/types.rs for apply module independence)
// =============================================================================

/// A single violation detected during analysis.
#[derive(Debug, Clone, Serialize)]
pub struct Violation {
    pub row: usize,
    pub message: String,
    pub law: &'static str,
    pub details: Option<ViolationDetails>,
}

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
            details: Some(details),
        }
    }
}

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

    #[must_use]
    pub fn clean_file_count(&self) -> usize {
        self.files.iter().filter(|f| f.is_clean()).count()
    }

    #[must_use]
    pub fn is_small_codebase(&self) -> bool {
        crate::analysis::Engine::small_codebase_threshold() >= self.files.len()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandResult {
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckReport {
    pub scan: ScanReport,
    pub commands: Vec<CommandResult>,
    pub passed: bool,
}

// =============================================================================
// Apply-Specific Types
// =============================================================================

/// A typed block from the XSC7XSC protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Plan(String),
    Manifest(String),
    Meta(String),
    File { path: String, content: String },
    Patch { path: String, content: String },
}

/// A single entry in the manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestEntry {
    pub path: String,
    pub operation: Operation,
}

/// Type alias for the manifest structure.
pub type Manifest = Vec<ManifestEntry>;

/// Operation type for a manifest entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    New,
    Update,
    Delete,
}

/// File content with metadata.
#[derive(Debug, Clone)]
pub struct FileContent {
    pub content: String,
    pub line_count: usize,
}

/// Extracted files mapped by path.
pub type ExtractedFiles = HashMap<String, FileContent>;

/// Input source for apply command.
#[derive(Debug, Clone)]
pub enum ApplyInput {
    Clipboard,
    Stdin,
    File(PathBuf),
}

/// Context for apply operations.
#[derive(Debug, Clone)]
pub struct ApplyContext {
    pub input: ApplyInput,
    pub repo_root: PathBuf,
    pub force: bool,
    pub dry_run: bool,
    pub reset_stage: bool,
    pub sanitize: bool,
    pub config: crate::config::Config,
    pub check_after: bool,
    pub auto_promote: bool,
    pub silent: bool,
}

impl ApplyContext {
    /// Creates a new `ApplyContext` with defaults from config.
    #[must_use]
    pub fn new(config: &crate::config::Config, repo_root: PathBuf) -> Self {
        Self {
            input: ApplyInput::Clipboard,
            repo_root,
            force: false,
            dry_run: false,
            reset_stage: false,
            sanitize: true,
            config: config.clone(),
            check_after: false,
            auto_promote: config.preferences.auto_promote,
            silent: false,
        }
    }
}

/// Outcome of an apply operation.
#[derive(Debug, Clone)]
pub enum ApplyOutcome {
    Success {
        written: Vec<String>,
        deleted: Vec<String>,
        backed_up: bool,
        staged: bool,
    },
    ValidationFailure {
        errors: Vec<String>,
        missing: Vec<String>,
        ai_message: String,
    },
    ParseError(String),
    WriteError(String),
    StageReset,
    Cancelled,
    Promoted {
        written: Vec<String>,
        deleted: Vec<String>,
    },
}
