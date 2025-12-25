// src/apply/processor.rs
use crate::apply::manifest;
use crate::apply::parser;
use crate::apply::types::{self, ApplyContext, ApplyOutcome, Block, FileContent, Operation};
use crate::apply::validator;
use crate::apply::verification;
use crate::apply::writer;
use crate::stage::StageManager;
use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};

/// Validates and applies a string payload containing a plan, manifest and files.
///
/// # Errors
/// Returns error if extraction, confirmation or writing fails.
pub fn process_input(content: &str, ctx: &ApplyContext) -> Result<ApplyOutcome> {
    if content.trim().is_empty() {
        return Ok(ApplyOutcome::ParseError("Input is empty".to_string()));
    }

    let blocks = match parser::parse(content) {
        Ok(b) => b,
        Err(e) => return Ok(ApplyOutcome::ParseError(format!("Parser Error: {e}"))),
    };

    if blocks.is_empty() {
        return Ok(ApplyOutcome::ParseError("No valid blocks found.".to_string()));
    }

    if let Some(outcome) = check_plan(&blocks, ctx)? {
        return Ok(outcome);
    }

    let (manifest, extracted) = extract_content(&blocks)?;

    warn_on_patch(&blocks);

    let validation = validator::validate(&manifest, &extracted);
    if !matches!(validation, ApplyOutcome::Success { .. }) {
        return Ok(validation);
    }

    apply_to_stage_transaction(&manifest, &extracted, ctx)
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

fn warn_on_patch(blocks: &[Block]) {
    if blocks.iter().any(|b| matches!(b, Block::Patch { .. })) {
        println!("{}", "WARN: PATCH blocks detected but currently ignored (Phase 2B pending).".yellow());
    }
}

fn check_plan_requirement(plan: Option<&str>, ctx: &ApplyContext) -> Result<bool> {
    if let Some(p) = plan {
        println!("{}", "[PLAN]:".cyan().bold());
        println!("{}", p.trim());
        if !ctx.force && !ctx.dry_run {
            return confirm("Apply these changes?");
        }
        return Ok(true);
    }
    if ctx.config.preferences.require_plan {
        println!("{}", "[X] REJECTED: Missing PLAN block.".red());
        Ok(false)
    } else {
        if !ctx.force && !ctx.dry_run {
            return confirm("Apply without a plan?");
        }
        Ok(true)
    }
}

fn apply_to_stage_transaction(
    manifest: &types::Manifest,
    extracted: &types::ExtractedFiles,
    ctx: &ApplyContext,
) -> Result<ApplyOutcome> {
    if ctx.dry_run {
        return Ok(ApplyOutcome::Success {
            written: vec!["(Dry Run) Verified".to_string()],
            deleted: vec![],
            backed_up: false,
            staged: false,
        });
    }

    let (mut stage, outcome) = execute_stage_transaction(manifest, extracted, ctx)?;

    if ctx.check_after {
        return run_post_apply_verification(ctx, &mut stage, outcome);
    }

    print_stage_info(&stage);
    Ok(outcome)
}

fn execute_stage_transaction(
    manifest: &types::Manifest,
    extracted: &types::ExtractedFiles,
    ctx: &ApplyContext,
) -> Result<(StageManager, ApplyOutcome)> {
    let mut stage = StageManager::new(&ctx.repo_root);
    let ensure_result = stage.ensure_stage()?;

    if ensure_result.was_created() {
        println!("{}", "Created staging workspace.".blue());
    }

    let worktree = stage.worktree();
    let retention = ctx.config.preferences.backup_retention;
    let outcome = writer::write_files(manifest, extracted, Some(&worktree), retention)?;

    for entry in manifest {
        match entry.operation {
            Operation::Delete => stage.record_delete(&entry.path)?,
            Operation::Update | Operation::New => stage.record_write(&entry.path)?,
        }
    }
    stage.record_apply()?;

    let staged_outcome = match outcome {
        ApplyOutcome::Success {
            written,
            deleted,
            backed_up,
            ..
        } => ApplyOutcome::Success {
            written,
            deleted,
            backed_up,
            staged: true,
        },
        other => other,
    };

    Ok((stage, staged_outcome))
}

fn run_post_apply_verification(
    ctx: &ApplyContext,
    stage: &mut StageManager,
    outcome: ApplyOutcome,
) -> Result<ApplyOutcome> {
    let passed = verification::run_verification_pipeline(ctx, stage.worktree())?;

    if passed {
        println!("{}", " Verification passed!".green().bold());
        if ctx.auto_promote {
            return promote_stage(ctx, stage);
        }
        if confirm("Promote staged changes to workspace?")? {
            return promote_stage(ctx, stage);
        }
        print_stage_info(stage);
        Ok(outcome)
    } else {
        println!("{}", "? Verification failed. Changes remain staged.".yellow());
        print_stage_info(stage);
        Ok(outcome)
    }
}

fn promote_stage(ctx: &ApplyContext, stage: &mut StageManager) -> Result<ApplyOutcome> {
    let retention = ctx.config.preferences.backup_retention;
    let result = stage.promote(retention)?;

    Ok(ApplyOutcome::Promoted {
        written: result.files_written,
        deleted: result.files_deleted,
    })
}

fn print_stage_info(stage: &StageManager) {
    println!(
        "\n{} Changes staged. Run {} to verify, or {} to promote.",
        " ".blue(),
        "slopchop check".yellow(),
        "slopchop apply --promote".yellow()
    );
    if let Some(state) = stage.state() {
        let write_count = state.paths_to_write().len();
        let delete_count = state.paths_to_delete().len();
        println!("   Stage: {write_count} writes, {delete_count} deletes pending");
    }
}

fn confirm(prompt: &str) -> Result<bool> {
    print!("{prompt} [y/N] ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}

/// Promotes staged changes to the real workspace (standalone command).
///
/// # Errors
/// Returns error if promotion fails.
pub fn run_promote_standalone(ctx: &ApplyContext) -> Result<ApplyOutcome> {
    let mut stage = StageManager::new(&ctx.repo_root);
    if !stage.exists() {
        println!("{}", "No stage to promote.".yellow());
        return Ok(ApplyOutcome::ParseError(
            "No staged changes found.".to_string(),
        ));
    }
    stage.load_state()?;
    promote_stage(ctx, &mut stage)
}