// src/apply/executor.rs
//! Handles the execution of apply actions: staging, verification, and promotion.

use crate::apply::types::{ApplyContext, ApplyOutcome, ExtractedFiles, Manifest, Operation};
use crate::apply::verification;
use crate::apply::writer;
use crate::events::{EventKind, EventLogger};
use crate::stage::StageManager;
use anyhow::Result;
use colored::Colorize;
use std::path::Path;
use std::io::{self, Write};

/// Executes the transaction to write changes to the stage.
///
/// # Errors
/// Returns error if stage operations or file writing fails.
pub fn apply_to_stage_transaction(
    manifest: &Manifest,
    extracted: &ExtractedFiles,
    ctx: &ApplyContext,
) -> Result<ApplyOutcome> {
    let logger = EventLogger::new(&ctx.repo_root);
    logger.log(EventKind::ApplyStarted);

    if ctx.dry_run {
        return Ok(ApplyOutcome::Success {
            written: vec!["(Dry Run) Verified".to_string()],
            deleted: vec![],
            backed_up: false,
            staged: false,
        });
    }

    let (mut stage, outcome) = execute_stage_transaction(manifest, extracted, ctx)?;

    // Log outcome
    match &outcome {
        ApplyOutcome::Success { written, deleted, .. } => {
            logger.log(EventKind::ApplySucceeded {
                files_written: written.len(),
                files_deleted: deleted.len(),
            });
        }
        ApplyOutcome::ParseError(e) | ApplyOutcome::WriteError(e) => {
            logger.log(EventKind::ApplyRejected { reason: e.clone() });
        }
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let reason = errors.join("; ");
            logger.log(EventKind::ApplyRejected { reason });
        }
        _ => {}
    }

    if ctx.check_after {
        return run_post_apply_verification(ctx, &mut stage, outcome);
    }

    print_stage_info(&stage);
    Ok(outcome)
}

fn execute_stage_transaction(
    manifest: &Manifest,
    extracted: &ExtractedFiles,
    ctx: &ApplyContext,
) -> Result<(StageManager, ApplyOutcome)> {
    let mut stage = StageManager::new(&ctx.repo_root);
    let ensure_result = stage.ensure_stage()?;

    if ensure_result.was_created() {
        println!("{}", "Created staging workspace.".blue());
    }

    let worktree = stage.worktree();
    let retention = ctx.config.preferences.backup_retention;
    
    // Write actual files to the shadow worktree
    let outcome = writer::write_files(manifest, extracted, Some(&worktree), retention)?;

    // Update stage tracking state
    update_stage_records(&mut stage, &ctx.repo_root, manifest)?;

    let staged_outcome = remap_staged_outcome(outcome);
    Ok((stage, staged_outcome))
}

fn update_stage_records(
    stage: &mut StageManager,
    repo_root: &Path,
    manifest: &Manifest,
) -> Result<()> {
    for entry in manifest {
        let base_hash = get_base_hash(repo_root, &entry.path)?;
        match entry.operation {
            Operation::Delete => stage.record_delete(&entry.path, base_hash)?,
            Operation::Update | Operation::New => stage.record_write(&entry.path, base_hash)?,
        }
    }
    stage.record_apply()
}

fn get_base_hash(repo_root: &Path, path: &str) -> Result<Option<String>> {
    let base_path = repo_root.join(path);
    if base_path.exists() && base_path.is_file() {
        let content = std::fs::read_to_string(&base_path)?;
        Ok(Some(crate::apply::patch::common::compute_sha256(&content)))
    } else {
        Ok(None)
    }
}

fn remap_staged_outcome(outcome: ApplyOutcome) -> ApplyOutcome {
    match outcome {
        ApplyOutcome::Success { written, deleted, backed_up, .. } => ApplyOutcome::Success {
            written,
            deleted,
            backed_up,
            staged: true,
        },
        other => other,
    }
}

fn run_post_apply_verification(
    ctx: &ApplyContext,
    stage: &mut StageManager,
    outcome: ApplyOutcome,
) -> Result<ApplyOutcome> {
    let result = verification::run_verification_pipeline(ctx, stage.worktree())?;

    if result.passed {
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
        
        let modified_files: Vec<String> = if let Some(state) = stage.state() {
             state.paths_to_write().iter().map(|p| p.path.clone()).collect()
        } else {
             Vec::new()
        };
        
        
        let ai_msg = verification::generate_ai_feedback(&result, &modified_files);
        crate::apply::messages::print_ai_feedback(&ai_msg);
        
        print_stage_info(stage);
        Ok(outcome)
    }
}

fn promote_stage(ctx: &ApplyContext, stage: &mut StageManager) -> Result<ApplyOutcome> {
    let logger = EventLogger::new(&ctx.repo_root);
    logger.log(EventKind::PromoteStarted);

    let retention = ctx.config.preferences.backup_retention;
    match stage.promote(retention) {
        Ok(result) => {
            logger.log(EventKind::PromoteSucceeded {
                files_written: result.files_written.len(),
                files_deleted: result.files_deleted.len(),
            });
            Ok(ApplyOutcome::Promoted {
                written: result.files_written,
                deleted: result.files_deleted,
            })
        }
        Err(e) => {
            logger.log(EventKind::PromoteFailed { error: e.to_string() });
            Err(e)
        }
    }
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

/// Prompts the user for confirmation (y/N).
///
/// # Errors
/// Returns error if reading from stdin fails.
pub fn confirm(prompt: &str) -> Result<bool> {
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