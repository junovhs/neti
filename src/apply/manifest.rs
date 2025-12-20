// src/apply/manifest.rs
use crate::apply::types::{ManifestEntry, Operation};
use anyhow::Result;
use regex::Regex;

const SIGIL: &str = "XSC7XSC";

/// Parses the delivery manifest block using the Sequence Sigil.
///
/// # Errors
/// Returns error if regex compilation fails.
pub fn parse_manifest(response: &str) -> Result<Option<Vec<ManifestEntry>>> {
    if let Some((start, end)) = find_sigil_manifest(response)? {
        let block = &response[start..end];
        let entries = parse_manifest_lines(block)?;
        return Ok(Some(entries));
    }
    Ok(None)
}

fn find_sigil_manifest(response: &str) -> Result<Option<(usize, usize)>> {
    let open_re = Regex::new(&format!(r"{SIGIL} MANIFEST {SIGIL}"))?;
    let close_re = Regex::new(&format!(r"{SIGIL} END {SIGIL}"))?;

    let Some(start_match) = open_re.find(response) else {
        return Ok(None);
    };

    let Some(end_match) = close_re.find_at(response, start_match.end()) else {
        return Ok(None);
    };

    Ok(Some((start_match.end(), end_match.start())))
}

fn parse_manifest_lines(block: &str) -> Result<Vec<ManifestEntry>> {
    let list_marker_re = Regex::new(r"^\s*(?:[-*]|\d+\.)\s+")?;
    let mut entries = Vec::new();

    for line in block.lines() {
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
