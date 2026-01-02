// src/apply/patch.rs
//! Surgical patch application logic.
//!
//! Supports:
//! - V1 (Canonical): Context-Anchored (`LEFT_CTX` / `OLD` / `RIGHT_CTX` / `NEW`).
//! - V0 (Deprecated): `SEARCH` / `REPLACE` blocks.

mod common;
mod diagnostics;
mod parser_v0;
mod parser_v1;

#[cfg(test)]
mod tests;

use self::common::{compute_sha256, detect_eol, normalize_newlines, PatchFormat, PatchInstruction};
use self::diagnostics::{diagnose_ambiguous, diagnose_zero_matches};
use anyhow::{anyhow, Result};

/// Applies a surgical patch to the original file content.
///
/// # Errors
/// Returns error if patch format is invalid, blocks not found, hash mismatch, or application fails.
pub fn apply(original: &str, patch_content: &str) -> Result<String> {
    let (instructions, metadata) = parse_patch(patch_content)?;

    if let Some(expected_hash) = metadata.base_sha256 {
        verify_hash(original, &expected_hash)?;
    } else if matches!(metadata.format, PatchFormat::V1) {
        return Err(anyhow!("Invalid V1 Patch: BASE_SHA256 is required."));
    }

    if matches!(metadata.format, PatchFormat::V0) {
        eprintln!("Warning: Deprecated V0 PATCH format. Please upgrade to V1.");
    }

    let mut current_content = original.to_string();
    for instr in instructions {
        current_content = apply_single_instruction(&current_content, &instr)?;
    }

    Ok(current_content)
}

struct PatchMetadata {
    base_sha256: Option<String>,
    format: PatchFormat,
}

fn parse_patch(content: &str) -> Result<(Vec<PatchInstruction>, PatchMetadata)> {
    let format = detect_format(content);
    match format {
        PatchFormat::V0 => parser_v0::parse(content).map(|(i, h)| (i, PatchMetadata {
            base_sha256: h,
            format: PatchFormat::V0,
        })),
        PatchFormat::V1 => parser_v1::parse(content).map(|(i, h)| (i, PatchMetadata {
            base_sha256: h,
            format: PatchFormat::V1,
        })),
    }
}

fn detect_format(content: &str) -> PatchFormat {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "<<<< SEARCH" {
            return PatchFormat::V0;
        }
        if trimmed == "LEFT_CTX:" {
            return PatchFormat::V1;
        }
    }
    PatchFormat::V1
}

fn verify_hash(content: &str, expected: &str) -> Result<()> {
    let actual = compute_sha256(content);
    if actual != *expected {
        return Err(anyhow!(
            "Patch mismatch: Base SHA256 verification failed.\n\
             Expected: {expected}\n\
             Actual:   {actual}\n\
             \n\
             NEXT: The file has changed since this patch was generated.\n\
             Run 'slopchop pack' to get the latest content and regenerate the patch."
        ));
    }
    Ok(())
}

fn apply_single_instruction(content: &str, instr: &PatchInstruction) -> Result<String> {
    let eol = detect_eol(content);
    let norm_search = normalize_newlines(&instr.search, eol);
    let norm_replace = normalize_newlines(&instr.replace, eol);

    let matches: Vec<_> = content.match_indices(&norm_search).collect();

    if matches.len() == 1 {
        return Ok(perform_splice(content, matches[0].0, &norm_search, &norm_replace));
    }

    if matches.len() > 1 {
        return Err(anyhow!(diagnose_ambiguous(matches.len(), &matches, content)));
    }

    // Zero matches: try without trailing EOL (handles files without final newline)
    try_match_trimmed(content, &norm_search, &norm_replace, eol, instr)
}

fn try_match_trimmed(
    content: &str,
    search: &str,
    replace: &str,
    eol: &str,
    instr: &PatchInstruction,
) -> Result<String> {
    let Some(trimmed_search) = search.strip_suffix(eol) else {
        return Err(anyhow!(diagnose_zero_matches(content, search, instr)));
    };

    let matches: Vec<_> = content.match_indices(trimmed_search).collect();

    match matches.len() {
        0 => Err(anyhow!(diagnose_zero_matches(content, search, instr))),
        1 => {
            let trimmed_replace = replace.strip_suffix(eol).unwrap_or(replace);
            Ok(perform_splice(content, matches[0].0, trimmed_search, trimmed_replace))
        }
        n => Err(anyhow!(diagnose_ambiguous(n, &matches, content))),
    }
}

fn perform_splice(content: &str, start: usize, search: &str, replace: &str) -> String {
    let end = start + search.len();
    let new_len = content.len() + replace.len().saturating_sub(search.len());
    let mut result = String::with_capacity(new_len);

    result.push_str(&content[..start]);
    result.push_str(replace);
    result.push_str(&content[end..]);

    result
}