// src/apply/manifest.rs
use crate::apply::types::{ManifestEntry, Operation};
use anyhow::Result;
use regex::Regex;

/// Parses the delivery manifest body lines.
///
/// # Errors
/// Returns error if regex compilation fails.
pub fn parse_manifest_body(content: &str) -> Result<Vec<ManifestEntry>> {
    let list_marker_re = Regex::new(r"^\s*(?:[-*]|\d+\.)\s+")?;
    let mut entries = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let clean_line = list_marker_re.replace(trimmed, "");
        let (path_raw, op) = parse_operation(clean_line.as_ref());
        let final_path = path_raw
            .split_whitespace()
            .next()
            .unwrap_or(&path_raw)
            .to_string();

        if !final_path.is_empty() {
            entries.push(ManifestEntry {
                path: final_path,
                operation: op,
            });
        }
    }
    Ok(entries)
}

fn parse_operation(line: &str) -> (String, Operation) {
    let upper = line.to_uppercase();
    if upper.contains("[NEW]") {
        (
            line.replace("[NEW]", "").replace("[new]", ""),
            Operation::New,
        )
    } else if upper.contains("[DELETE]") {
        (
            line.replace("[DELETE]", "").replace("[delete]", ""),
            Operation::Delete,
        )
    } else {
        (line.to_string(), Operation::Update)
    }
}

/// Deprecated: kept temporarily if needed by old tests, but prefer `parse_manifest_body`.
///
/// # Errors
/// Always returns `Ok(None)`.
pub fn parse_manifest(_response: &str) -> Result<Option<Vec<ManifestEntry>>> {
    // This function is now legacy as the parser handles block extraction.
    // It's stubbed out or redirected to ensure type compatibility if called incorrectly.
    Ok(None)
}