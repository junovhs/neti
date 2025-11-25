// src/apply/extractor.rs
use crate::apply::types::FileContent;
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Extracts file blocks from the raw response text.
///
/// # Errors
/// Returns error if regex compilation fails.
pub fn extract_files(response: &str) -> Result<HashMap<String, FileContent>> {
    let mut files = HashMap::new();

    let open_tag_re = Regex::new(r"(?i)<file\s+([^>]+)>")?;
    let path_attr_re = Regex::new(r#"(?i)path\s*=\s*(?:"([^"]*)"|'([^']*)'|([^>\s]+))"#)?;
    let close_tag_re = Regex::new(r"(?i)</file>")?;

    let mut current_pos = 0;

    while let Some(cap) = open_tag_re.find_at(response, current_pos) {
        let _tag_start = cap.start();
        let tag_end = cap.end();
        let attributes_str = cap.as_str();

        let path = if let Some(captures) = path_attr_re.captures(attributes_str) {
            captures
                .get(1)
                .or_else(|| captures.get(2))
                .or_else(|| captures.get(3))
                .map(|m| m.as_str().to_string())
        } else {
            current_pos = tag_end;
            continue;
        };

        if let Some(file_path) = path {
            if let Some(close_match) = close_tag_re.find_at(response, tag_end) {
                let content_end = close_match.start();
                let raw_content = &response[tag_end..content_end];

                let clean_content = clean_file_content(raw_content);
                let line_count = clean_content.lines().count();

                files.insert(
                    file_path,
                    FileContent {
                        content: clean_content,
                        line_count,
                    },
                );

                current_pos = close_match.end();
            } else {
                break;
            }
        } else {
            current_pos = tag_end;
        }
    }

    Ok(files)
}

fn clean_file_content(raw: &str) -> String {
    let trimmed = raw.trim();
    let lines: Vec<&str> = trimmed.lines().collect();

    if lines.is_empty() {
        return String::new();
    }

    let first_line = lines.first().unwrap_or(&"").trim();
    let last_line = lines.last().unwrap_or(&"").trim();

    let starts_with_fence = first_line.starts_with("```");
    let ends_with_fence = last_line.starts_with("```");

    if starts_with_fence && ends_with_fence {
        if lines.len() <= 2 {
            return String::new();
        }
        return lines[1..lines.len() - 1].join("\n");
    }

    trimmed.to_string()
}
