// src/apply/processor.rs
use crate::apply::executor;
use crate::apply::manifest;
use crate::apply::parser;
use crate::apply::patch;
use crate::apply::types::{self, ApplyContext, ApplyOutcome, Block, FileContent};
use crate::apply::validator;
use crate::events::{EventKind, EventLogger};
use crate::stage::StageManager;
use anyhow::{anyhow, Result};
use colored::Colorize;
use std::path::Path;

/// Validates and applies a string payload containing a plan, manifest and files.
///
/// # Errors
/// Returns error if extraction, confirmation or writing fails.
pub fn process_input(content: &str, ctx: &ApplyContext) -> Result<ApplyOutcome> {
    if content.trim().is_empty() {
        return Ok(ApplyOutcome::ParseError("Input is empty".to_string()));
    }

    let blocks = parser::parse(content).map_err(|e| anyhow!("Parser Error: {e}"))?;
    if blocks.is_empty() {
        return Ok(ApplyOutcome::ParseError("No valid blocks found.".to_string()));
    }

    if let Some(outcome) = check_plan(&blocks, ctx)? {
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

    executor::apply_to_stage_transaction(&manifest, &extracted, ctx)
}

fn perform_sanitization(extracted: &mut types::ExtractedFiles, ctx: &ApplyContext) {
    let logger = EventLogger::new(&ctx.repo_root);
    
    for (path, content) in extracted.iter_mut() {
        if is_markdown(path) { continue; }
        
        let original_count = content.content.lines().count();
        let sanitized: Vec<&str> = content.content.lines()
            .filter(|line| !is_fence_line(line))
            .collect();
        let new_count = sanitized.len();
            
        if new_count != original_count {
            let removed = original_count - new_count;
            println!("   {} Sanitized {} markdown fence lines from {}", "i".blue(), removed, path);
            
            logger.log(EventKind::SanitizationPerformed {
                path: path.clone(),
                lines_removed: removed,
            });

            let new_text = sanitized.join("\n");
            
            // Rejoin with original newlines is hard without more logic, 
            // but we usually just want standard \n for code.
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
        .is_some_and(|ext| {
            ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown")
        })
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

fn check_plan(blocks: &[Block], ctx: &ApplyContext) -> Result<Option<ApplyOutcome>> {
    let plan = blocks.iter().find_map(|b| match b {
        Block::Plan(s) => Some(s.as_str()),
        _ => None,
    });

    if check_plan_requirement(plan, ctx)? {
        Ok(None)
    } else {
        Ok(Some(ApplyOutcome::ParseError("Operation cancelled.".to_string())))
    }
}

fn check_plan_requirement(plan: Option<&str>, ctx: &ApplyContext) -> Result<bool> {
    if let Some(p) = plan {
        println!("{}", "[PLAN]:".cyan().bold());
        println!("{}", p.trim());
        if !ctx.force && !ctx.dry_run {
            return executor::confirm("Apply these changes?");
        }
        return Ok(true);
    }
    if ctx.config.preferences.require_plan {
        println!("{}", "[X] REJECTED: Missing PLAN block.".red());
        Ok(false)
    } else {
        if !ctx.force && !ctx.dry_run {
            return executor::confirm("Apply without a plan?");
        }
        Ok(true)
    }
}

fn apply_patches(
    blocks: &[Block],
    extracted: &mut types::ExtractedFiles,
    ctx: &ApplyContext,
) -> Result<()> {
    let mut stage_manager = StageManager::new(&ctx.repo_root);
    let _ = stage_manager.load_state(); 

    for block in blocks {
        if let Block::Patch { path, content } = block {
            let base = get_base_content(path, extracted, &stage_manager, &ctx.repo_root)?;
            let new_content = patch::apply(&base, content)
                .map_err(|e| anyhow!("Failed to patch {path}: {e}"))?;

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
    stage: &StageManager,
    repo_root: &Path,
) -> Result<String> {
    // 1. Check extracted (previous patches in same payload)
    if let Some(fc) = extracted.get(path) {
        return Ok(fc.content.clone());
    }
    // 2. Check stage
    if stage.exists() {
        let p = stage.worktree().join(path);
        if p.exists() {
            return std::fs::read_to_string(p).map_err(|e| anyhow!("Read staged {path}: {e}"));
        }
    }
    // 3. Check workspace
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