// src/apply/types.rs
use crate::config::Config;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    Update,
    New,
    Delete,
}

#[derive(Debug, Clone)]
pub struct ManifestEntry {
    pub path: String,
    pub operation: Operation,
}

#[derive(Debug, Clone)]
pub struct FileContent {
    pub content: String,
    pub line_count: usize,
}

#[derive(Debug)]
pub enum ApplyOutcome {
    Success {
        written: Vec<String>,
        backed_up: bool,
    },
    ValidationFailure {
        errors: Vec<String>,
        missing: Vec<String>,
        ai_message: String,
    },
    ParseError(String),
    WriteError(String),
}

/// Context for the apply operation.
/// Connects project config with runtime flags.
pub struct ApplyContext<'a> {
    pub config: &'a Config,
    pub force: bool,   // Skips interactive confirmation (for tests/automation)
    pub dry_run: bool, // Skips disk writes (for tests)
}

impl<'a> ApplyContext<'a> {
    #[must_use]
    pub fn new(config: &'a Config) -> Self {
        Self {
            config,
            force: false,
            dry_run: false,
        }
    }
}

// The manifest is just a list of entries
pub type Manifest = Vec<ManifestEntry>;

// The extracted files are mapped by path
pub type ExtractedFiles = HashMap<String, FileContent>;