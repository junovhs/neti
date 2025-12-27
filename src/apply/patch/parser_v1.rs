// src/apply/patch/parser_v1.rs
//! Canonical V1 patch parser (`LEFT_CTX` / `OLD` / `RIGHT_CTX` / `NEW`).

use super::common::{collect_until_keyword, PatchInstruction};
use anyhow::{anyhow, Result};

const KEYWORDS: &[&str] = &["LEFT_CTX:", "OLD:", "RIGHT_CTX:", "NEW:", "XSC7XSC END"];

pub fn parse(content: &str) -> Result<(Vec<PatchInstruction>, Option<String>)> {
    let mut ctx = ParseContext {
        base_sha256: None,
        instructions: Vec::new(),
        
        left_ctx: None,
        old: None,
        right_ctx: None,
        new_val: None,
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        i = process_line(lines[i], &lines, i, &mut ctx)?;
    }

    // Check if we have leftover partial blocks (error state)
    // Only error if it looks like they tried to write a patch (has LEFT_CTX) but got nothing
    if ctx.instructions.is_empty() && content.contains("LEFT_CTX:") {
         return Err(anyhow!("Parsed V1 headers but found no complete instruction set. Ensure all parts are present."));
    }

    Ok((ctx.instructions, ctx.base_sha256))
}

struct ParseContext {
    base_sha256: Option<String>,
    instructions: Vec<PatchInstruction>,
    
    left_ctx: Option<String>,
    old: Option<String>,
    right_ctx: Option<String>,
    new_val: Option<String>,
}

fn process_line(
    line: &str,
    all_lines: &[&str],
    idx: usize,
    ctx: &mut ParseContext,
) -> Result<usize> {
    let trimmed = line.trim();

    // Use trimmed line for metadata to tolerate indentation
    if let Some(val) = trimmed.strip_prefix("BASE_SHA256:") {
        ctx.base_sha256 = Some(val.trim().to_string());
        return Ok(idx + 1);
    }
    
    if let Some(val) = trimmed.strip_prefix("MAX_MATCHES:") {
        validate_max_matches(val.trim())?;
        return Ok(idx + 1);
    }

    if is_block_header(trimmed) {
        return Ok(parse_block_content(trimmed, all_lines, idx, ctx));
    }

    Ok(idx + 1)
}

fn validate_max_matches(val: &str) -> Result<()> {
    if val == "1" {
        Ok(())
    } else {
        Err(anyhow!("V1 Protocol Violation: MAX_MATCHES must be 1. Got: {val}"))
    }
}

fn is_block_header(line: &str) -> bool {
    matches!(line, "LEFT_CTX:" | "OLD:" | "RIGHT_CTX:" | "NEW:")
}

fn parse_block_content(
    header: &str,
    lines: &[&str],
    idx: usize,
    ctx: &mut ParseContext,
) -> usize {
    let (text, next_idx) = collect_until_keyword(lines, idx + 1, KEYWORDS);

    match header {
        "LEFT_CTX:" => ctx.left_ctx = Some(text),
        "OLD:" => ctx.old = Some(text),
        "RIGHT_CTX:" => ctx.right_ctx = Some(text),
        "NEW:" => ctx.new_val = Some(text),
        _ => {}
    }

    try_build_instruction(ctx);
    next_idx
}

fn try_build_instruction(ctx: &mut ParseContext) {
    // Only build if we have all four components.
    if ctx.left_ctx.is_none() 
        || ctx.old.is_none() 
        || ctx.right_ctx.is_none() 
        || ctx.new_val.is_none() 
    {
        return;
    }

    if let (Some(l), Some(o), Some(r), Some(n)) = (
        ctx.left_ctx.take(),
        ctx.old.take(),
        ctx.right_ctx.take(),
        ctx.new_val.take(),
    ) {
        let search = format!("{l}{o}{r}");
        let replace = format!("{l}{n}{r}");

        ctx.instructions.push(PatchInstruction {
            search,
            replace,
            context_left: Some(l),
        });
    } else {
        unreachable!("Components verified present but failed to take");
    }
}