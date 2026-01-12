// src/apply/patch/parser_v0.rs
//! Legacy V0 patch parser (`<<<< SEARCH` ... `====` ... `>>>>`).

use super::common::{collect_section, PatchInstruction};
use anyhow::{anyhow, Result};

/// Parses a legacy V0 patch payload.
///
/// # Errors
/// Returns error if no valid SEARCH blocks are found or if the patch is malformed.
pub fn parse(content: &str) -> Result<(Vec<PatchInstruction>, Option<String>)> {
    let mut instructions = Vec::new();
    let mut base_sha256 = None;

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while let Some(line) = lines.get(i) {
        if let Some(stripped) = line.strip_prefix("BASE_SHA256:") {
            base_sha256 = Some(stripped.trim().to_string());
            i += 1;
            continue;
        }

        if line.trim() == "<<<< SEARCH" {
            let (instr, next_i) = parse_block(&lines, i)?;
            instructions.push(instr);
            i = next_i;
        } else {
            i += 1;
        }
    }

    if instructions.is_empty() {
        return Err(anyhow!("No valid <<<< SEARCH blocks found in V0 patch."));
    }

    Ok((instructions, base_sha256))
}

fn parse_block(lines: &[&str], start_index: usize) -> Result<(PatchInstruction, usize)> {
    let (search_text, i) = collect_section(
        lines,
        start_index + 1,
        "====",
        "Missing '====' separator after SEARCH block",
    )?;

    let (replace_text, i) = collect_section(
        lines,
        i,
        ">>>>",
        "Missing '>>>>' terminator after REPLACE block",
    )?;

    Ok((
        PatchInstruction {
            search: search_text,
            replace: replace_text,
            context_left: None,
        },
        i,
    ))
}
