// src/apply/manifest.rs
use crate::apply::types::{ManifestEntry, Operation};
use anyhow::Result;
use regex::Regex;

/// Parses the delivery manifest block.
///
/// # Errors
/// Returns error if regex compilation fails.
pub fn parse_manifest(response: &str) -> Result<Option<Vec<ManifestEntry>>> {
    let open_re = Regex::new(r"(?i)<delivery>")?;
    let close_re = Regex::new(r"(?i)</delivery>")?;

    let start_match = open_re.find(response);
    let end_match = close_re.find(response);

    if let (Some(start), Some(end)) = (start_match, end_match) {
        let block = &response[start.end()..end.start()];
        let mut entries = Vec::new();

        for line in block.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let clean_line = trimmed
                .trim_start_matches('-')
                .trim_start_matches('*')
                .trim_start_matches(|c: char| c.is_ascii_digit())
                .trim_start_matches('.')
                .trim();

            if clean_line.is_empty() {
                continue;
            }

            let (path, op) = if clean_line.to_uppercase().contains("[NEW]") {
                (
                    clean_line.replace("[NEW]", "").replace("[new]", ""),
                    Operation::New,
                )
            } else if clean_line.to_uppercase().contains("[DELETE]") {
                (
                    clean_line.replace("[DELETE]", "").replace("[delete]", ""),
                    Operation::Delete,
                )
            } else {
                (clean_line.to_string(), Operation::Update)
            };

            let final_path = path
                .split_whitespace()
                .next()
                .unwrap_or(&path)
                .trim()
                .to_string();

            if !final_path.is_empty() {
                entries.push(ManifestEntry {
                    path: final_path,
                    operation: op,
                });
            }
        }

        Ok(Some(entries))
    } else {
        Ok(None)
    }
}
