// src/apply/patch/common.rs
//! Shared types and utilities for patch parsing and application.

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy)]
pub enum PatchFormat {
    V0,
    V1,
}

#[derive(Debug)]
pub struct PatchInstruction {
    pub search: String,
    pub replace: String,
    // Store components for diagnostics
    pub context_left: Option<String>,
}

#[must_use]
pub fn detect_eol(content: &str) -> &str {
    if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

#[must_use]
pub fn normalize_newlines(text: &str, eol: &str) -> String {
    // robust normalization that preserves trailing newlines and blank lines
    let lf_only = text.replace("\r\n", "\n").replace('\r', "\n");
    if eol == "\n" {
        lf_only
    } else {
        lf_only.replace('\n', eol)
    }
}

#[must_use]
pub fn compute_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

/// Collects lines until a specific keyword is found.
/// Used by both parsers for section extraction.
#[must_use]
pub fn collect_until_keyword(lines: &[&str], start: usize, keywords: &[&str]) -> (String, usize) {
    let mut collected = Vec::new();
    let mut i = start;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        
        if keywords.iter().any(|k| trimmed.starts_with(k)) {
            break;
        }
        collected.push(line);
        i += 1;
    }
    
    // Join with simple \n. Normalization happens at application time.
    let mut joined = collected.join("\n");
    if !joined.is_empty() {
        joined.push('\n');
    }
    
    (joined, i)
}

/// Collects a strictly delimited section (V0 style).
///
/// # Errors
/// Returns error if the terminator is not found.
pub fn collect_section(
    lines: &[&str],
    start_index: usize,
    terminator: &str,
    err_msg: &str,
) -> Result<(String, usize)> {
    let mut i = start_index;
    let mut collected = Vec::new();
    let mut found = false;

    while i < lines.len() {
        let line = lines[i];
        if line.trim() == terminator {
            found = true;
            i += 1;
            break;
        }
        collected.push(line);
        i += 1;
    }

    if !found {
        return Err(anyhow!("{err_msg}"));
    }

    Ok((collected.join("\n"), i))
}