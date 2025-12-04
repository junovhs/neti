// src/apply/manifest.rs
use crate::apply::types::{ManifestEntry, Operation};
use anyhow::Result;
use regex::Regex;

/// Parses the delivery manifest block.
/// Supports both Legacy XML and SlopChop Protocol.
///
/// # Errors
/// Returns error if regex compilation fails.
pub fn parse_manifest(response: &str) -> Result<Option<Vec<ManifestEntry>>> {
    if let Some((start, end)) = find_slopchop_manifest(response)? {
        let block = &response[start..end];
        let entries = parse_manifest_lines(block)?;
        return Ok(Some(entries));
    }

    if let Some((start, end)) = find_legacy_manifest(response)? {
        let block = &response[start..end];
        let entries = parse_manifest_lines(block)?;
        return Ok(Some(entries));
    }

    Ok(None)
}

fn find_slopchop_manifest(response: &str) -> Result<Option<(usize, usize)>> {
    let open_re = Regex::new(r"#__SLOPCHOP_MANIFEST__#")?;
    let close_re = Regex::new(r"#__SLOPCHOP_END__#")?;

    let Some(start_match) = open_re.find(response) else {
        return Ok(None);
    };

    let Some(end_match) = close_re.find_at(response, start_match.end()) else {
        return Ok(None);
    };

    Ok(Some((start_match.end(), end_match.start())))
}

fn find_legacy_manifest(response: &str) -> Result<Option<(usize, usize)>> {
    let open_re = Regex::new(r"(?i)<delivery>")?;
    let close_re = Regex::new(r"(?i)</delivery>")?;

    let start_match = open_re.find(response);
    let end_match = close_re.find(response);

    match (start_match, end_match) {
        (Some(s), Some(e)) => Ok(Some((s.end(), e.start()))),
        _ => Ok(None),
    }
}

fn parse_manifest_lines(block: &str) -> Result<Vec<ManifestEntry>> {
    let list_marker_re = Regex::new(r"^\s*(?:[-*]|\d+\.)\s+")?;
    let mut entries = Vec::new();

    for line in block.lines() {
        if let Some(entry) = parse_manifest_line(line, &list_marker_re) {
            entries.push(entry);
        }
    }

    Ok(entries)
}

fn parse_manifest_line(line: &str, marker_re: &Regex) -> Option<ManifestEntry> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let clean_line = marker_re.replace(trimmed, "");
    let clean_line_ref = clean_line.as_ref();

    if clean_line_ref.trim().is_empty() {
        return None;
    }

    let (path_raw, op) = parse_operation(clean_line_ref);
    let final_path = extract_clean_path(&path_raw);

    if final_path.is_empty() {
        None
    } else {
        Some(ManifestEntry {
            path: final_path,
            operation: op,
        })
    }
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

fn extract_clean_path(raw: &str) -> String {
    raw.split_whitespace().next().unwrap_or(raw).to_string()
}
