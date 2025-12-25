// src/apply/executor.rs
//! Handles the execution of apply actions: staging, verification, and promotion.

use crate::apply::types::{ApplyContext, ApplyOutcome, ExtractedFiles, Manifest, Operation};
use crate::apply::verification;
use crate::apply::writer;
use crate::stage::StageManager;
use anyhow::Result;
use colored::Colorize;
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
    for entry in manifest {
        match entry.operation {
            Operation::Delete => stage.record_delete(&entry.path)?,
            Operation::Update | Operation::New => stage.record_write(&entry.path)?,
        }
    }
    stage.record_apply()?;

    let staged_outcome = match outcome {
        ApplyOutcome::Success { written, deleted, backed_up, .. } => ApplyOutcome::Success {
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