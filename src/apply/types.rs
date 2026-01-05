// src/apply/types.rs
use crate::config::Config;
use std::collections::HashMap;
use std::path::PathBuf;

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

/// Represents a parsed block from the `SlopChop` protocol stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Plan(String),
    Manifest(String),
    File { path: String, content: String },
    Patch { path: String, content: String },
    Meta(String),
}

#[derive(Debug)]
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
    Promoted {
        written: Vec<String>,
        deleted: Vec<String>,
    },
    StageReset,
}

#[derive(Debug, Clone, Default)]
pub enum ApplyInput {
    #[default]
    Clipboard,
    Stdin,
    File(PathBuf),
}

#[allow(clippy::struct_excessive_bools)]
pub struct ApplyContext<'a> {
    pub config: &'a Config,
    pub repo_root: PathBuf,
    pub force: bool,
    pub dry_run: bool,
    pub input: ApplyInput,
    pub check_after: bool,
    pub auto_promote: bool,
    pub reset_stage: bool,
    pub sanitize: bool,
    pub silent: bool,
}

impl<'a> ApplyContext<'a> {
    #[must_use]
    pub fn new(config: &'a Config, repo_root: PathBuf) -> Self {
        Self {
            config,
            repo_root,
            force: false,
            dry_run: false,
            input: ApplyInput::default(),
            check_after: false,
            auto_promote: false,
            reset_stage: false,
            sanitize: true, // Default safe for clipboard
            silent: false,
        }
    }
}

pub type Manifest = Vec<ManifestEntry>;
pub type ExtractedFiles = HashMap<String, FileContent>;