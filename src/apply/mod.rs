// src/apply/mod.rs
pub mod extractor;
pub mod manifest;
pub mod messages;
pub mod types;
pub mod validator;
pub mod writer;

use crate::clipboard;
use anyhow::{Context, Result};
use types::ApplyOutcome;

/// Runs the apply command logic.
///
/// # Errors
/// Returns error if clipboard access fails or extraction fails.
pub fn run_apply(dry_run: bool) -> Result<ApplyOutcome> {
    // 1. Read Clipboard
    let content = clipboard::read_clipboard().context("Failed to read clipboard")?;

    if content.trim().is_empty() {
        return Ok(ApplyOutcome::ParseError("Clipboard is empty".to_string()));
    }

    // 2. Parse & Validate
    let validation = parse_and_validate(&content);

    // 3. Execute (if valid)
    match validation {
        ApplyOutcome::Success { .. } => {
            if dry_run {
                return Ok(ApplyOutcome::Success {
                    written: vec!["(Dry Run) Files verified".to_string()],
                    backed_up: false,
                });
            }
            // Fixed: pass reference to writer::write_files
            writer::write_files(&extractor::extract_files(&content)?)
        }
        _ => Ok(validation),
    }
}

/// Runs the undo command.
///
/// # Errors
/// Returns error if backup restore fails.
pub fn run_undo() -> Result<Vec<std::path::PathBuf>> {
    writer::restore_backup()
}

pub fn print_result(outcome: &ApplyOutcome) {
    messages::print_outcome(outcome);
}

fn parse_and_validate(content: &str) -> ApplyOutcome {
    // 1. Parse Manifest
    let manifest_result = manifest::parse_manifest(content);
    let manifest = match manifest_result {
        Ok(Some(m)) => m,
        Ok(None) => Vec::new(),
        Err(e) => return ApplyOutcome::ParseError(format!("Manifest Error: {e}")),
    };

    // 2. Extract Files
    let extracted_result = extractor::extract_files(content);
    let extracted = match extracted_result {
        Ok(e) => e,
        Err(e) => return ApplyOutcome::ParseError(format!("Extraction Error: {e}")),
    };

    // 3. Validate
    validator::validate(&manifest, &extracted)
}
