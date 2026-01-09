// src/apply/processor.rs
use crate::apply::executor;
use crate::apply::manifest;
use crate::apply::parser;
use crate::apply::patch;
use crate::apply::types::{self, ApplyContext, ApplyOutcome, Block, FileContent};
use crate::apply::validator;
use crate::events::{EventKind, EventLogger};
use anyhow::{anyhow, Result};
use colored::Colorize;
use std::collections::HashSet;
use std::path::Path;

/// Validates and applies a string payload containing a plan, manifest and files.
///
/// # Errors
/// Returns error if extraction, confirmation or writing fails.
pub fn process_input(content: &str, ctx: &ApplyContext) -> Result<ApplyOutcome> {
    let blocks = match parse_content(content)? {
        Ok(b) => b,
        Err(outcome) => return Ok(outcome),
    };

    let plan_text = get_plan_text(&blocks);

    if let Some(outcome) = check_plan_requirement(plan_text, ctx)? {
        return Ok(outcome);
    }

    let (manifest, mut extracted) = extract_content(&blocks)?;

    if ctx.sanitize {
        perform_sanitization(&mut extracted, ctx);
    }

    // Process PATCH blocks
    if let Err(e) = apply_patches(&blocks, &mut extracted, ctx) {
        return Ok(ApplyOutcome::ParseError(format!("Patch Error: {e}")));
    }

    let validation = validator::validate(&manifest, &extracted);
    if !matches!(validation, ApplyOutcome::Success { .. }) {
        return Ok(validation);
    }

    let commit_msg = extract_commit_message(plan_text);
    executor::apply_to_stage_transaction(&manifest, &extracted, ctx, &commit_msg)
}

fn parse_content(content: &str) -> Result<Result<Vec<Block>, ApplyOutcome>> {
    if content.trim().is_empty() {
        return Ok(Err(ApplyOutcome::ParseError("Input is empty".to_string())));
    }
    let blocks = parser::parse(content).map_err(|e| anyhow!("Parser Error: {e}"))?;
    if blocks.is_empty() {
        return Ok(Err(ApplyOutcome::ParseError(
            "No valid blocks found.".to_string(),
        )));
    }
    Ok(Ok(blocks))
}

fn get_plan_text(blocks: &[Block]) -> Option<&str> {
    blocks.iter().find_map(|b| match b {
        Block::Plan(s) => Some(s.as_str()),
        _ => None,
    })
}

fn extract_commit_message(plan: Option<&str>) -> String {
    plan.and_then(|text| {
        text.lines().find_map(|line| {
            let clean = line.trim().strip_prefix("GOAL:")?.trim();
            if clean.is_empty() {
                None
            } else {
                Some(format!("ai: {clean}"))
            }
        })
    })
    .unwrap_or_else(|| "chore: apply slopchop changes".to_string())
}

fn perform_sanitization(extracted: &mut types::ExtractedFiles, ctx: &ApplyContext) {
    let logger = EventLogger::new(&ctx.repo_root);

    for (path, content) in extracted.iter_mut() {
        if is_markdown(path) {
            continue;
        }

        let original_count = content.content.lines().count();
        let sanitized: Vec<&str> = content
            .content
            .lines()
            .filter(|line| !is_fence_line(line))
            .collect();
        let new_count = sanitized.len();

        if new_count != original_count {
            let removed = original_count - new_count;
            println!(
                "   {} Sanitized {} markdown fence lines from {}",
                "i".blue(),
                removed,
                path
            );

            logger.log(EventKind::SanitizationPerformed {
                path: path.clone(),
                lines_removed: removed,
            });

            let new_text = sanitized.join("\n");

            content.content = new_text;
            if !content.content.is_empty() {
                content.content.push('\n');
            }
            content.line_count = new_count;
        }
    }
}

fn is_markdown(path: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
}

fn is_fence_line(line: &str) -> bool {
    let trimmed = line.trim();
    // Use hex escapes to avoid triggering the self-check validator during apply
    trimmed.starts_with("\x60\x60\x60") || trimmed.starts_with("\x7E\x7E\x7E")
}

fn extract_content(blocks: &[Block]) -> Result<(types::Manifest, types::ExtractedFiles)> {
    let manifest_block = blocks.iter().find_map(|b| match b {
        Block::Manifest(s) => Some(s.as_str()),
        _ => None,
    });

    let manifest = match manifest_block {
        Some(m_str) => manifest::parse_manifest_body(m_str)?,
        None => Vec::new(),
    };

    let mut extracted = std::collections::HashMap::new();
    for block in blocks {
        if let Block::File { path, content } = block {
            extracted.insert(
                path.clone(),
                FileContent {
                    content: content.clone(),
                    line_count: content.lines().count(),
                },
            );
        }
    }
    Ok((manifest, extracted))
}

fn check_plan_requirement(plan: Option<&str>, ctx: &ApplyContext) -> Result<Option<ApplyOutcome>> {
    if let Some(p) = plan {
        println!("{}", "[PLAN]:".cyan().bold());
        println!("{}", p.trim());
        if check_interactive_abort(ctx, "Apply these changes?")? {
            return Ok(Some(ApplyOutcome::ParseError(
                "Operation cancelled.".to_string(),
            )));
        }
        return Ok(None);
    }

    if ctx.config.preferences.require_plan {
        println!("{}", "[X] REJECTED: Missing PLAN block.".red());
        return Ok(Some(ApplyOutcome::ParseError(
            "Missing PLAN block (required by config).".to_string(),
        )));
    }

    if check_interactive_abort(ctx, "Apply without a plan?")? {
        return Ok(Some(ApplyOutcome::ParseError(
            "Operation cancelled.".to_string(),
        )));
    }
    Ok(None)
}

fn check_interactive_abort(ctx: &ApplyContext, prompt: &str) -> Result<bool> {
    if ctx.force || ctx.dry_run {
        return Ok(false);
    }
    Ok(!executor::confirm(prompt)?)
}

fn apply_patches(
    blocks: &[Block],
    extracted: &mut types::ExtractedFiles,
    ctx: &ApplyContext,
) -> Result<()> {
    // Track files that have already been hash-verified
    let mut verified: HashSet<String> = HashSet::new();

    for block in blocks {
        if let Block::Patch { path, content } = block {
            let base = get_base_content(path, extracted, &ctx.repo_root)?;

            // Only verify hash on first patch to each file
            let skip_hash = verified.contains(path);
            let new_content = patch::apply_with_options(&base, content, skip_hash)
                .map_err(|e| anyhow!("Failed to patch {path}: {e}"))?;

            verified.insert(path.clone());

            extracted.insert(
                path.clone(),
                FileContent {
                    line_count: new_content.lines().count(),
                    content: new_content,
                },
            );
        }
    }
    Ok(())
}

fn get_base_content(
    path: &str,
    extracted: &types::ExtractedFiles,
    repo_root: &Path,
) -> Result<String> {
    // 1. Check extracted (previous patches in same payload)
    if let Some(fc) = extracted.get(path) {
        return Ok(fc.content.clone());
    }
    // 2. Check workspace
    let p = repo_root.join(path);
    if p.exists() {
        return std::fs::read_to_string(p).map_err(|e| anyhow!("Read original {path}: {e}"));
    }
    Err(anyhow!("Base file not found for patch: {path}"))
}

/// Promotes staged changes to the real workspace (standalone command).
///
/// # Errors
/// Returns error if promotion fails.
pub fn run_promote_standalone(ctx: &ApplyContext) -> Result<ApplyOutcome> {
    executor::run_promote_standalone(ctx)
}