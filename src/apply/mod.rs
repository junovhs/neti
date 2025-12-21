// src/apply/mod.rs
pub mod backup;
pub mod extractor;
pub mod manifest;
pub mod messages;
pub mod types;
pub mod validator;
pub mod verification;
pub mod writer;

use crate::clipboard;
use crate::stage::StageManager;
use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, Read, Write};
use types::{ApplyContext, ApplyInput, ApplyOutcome, Operation};

/// Executes the apply operation based on user input.
///
/// # Errors
/// Returns error if input reading or processing fails.
pub fn run_apply(ctx: &ApplyContext) -> Result<ApplyOutcome> {
    if ctx.reset_stage {
        return reset_stage(ctx);
    }
    let content = read_input(&ctx.input)?;
    process_input(&content, ctx)
}

fn reset_stage(ctx: &ApplyContext) -> Result<ApplyOutcome> {
    let mut stage = StageManager::new(&ctx.repo_root);
    if !stage.exists() {
        println!("{}", "No stage to reset.".yellow());
        return Ok(ApplyOutcome::StageReset);
    }
    stage.reset()?;
    println!("{}", "Stage reset successfully.".green());
    Ok(ApplyOutcome::StageReset)
}

fn read_input(input: &ApplyInput) -> Result<String> {
    match input {
        ApplyInput::Clipboard => clipboard::read_clipboard().context("Failed to read clipboard"),
        ApplyInput::Stdin => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).context("Failed to read stdin")?;
            Ok(buf)
        }
        ApplyInput::File(path) => std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display())),
    }
}

pub fn print_result(outcome: &ApplyOutcome) {
    messages::print_outcome(outcome);
}

/// Validates and applies a string payload containing a plan, manifest and files.
///
/// # Errors
/// Returns error if extraction, confirmation or writing fails.
pub fn process_input(content: &str, ctx: &ApplyContext) -> Result<ApplyOutcome> {
    if content.trim().is_empty() {
        return Ok(ApplyOutcome::ParseError("Input is empty".to_string()));
    }
    let plan_opt = extractor::extract_plan(content);
    if !check_plan_requirement(plan_opt.as_deref(), ctx)? {
        return Ok(ApplyOutcome::ParseError("Operation cancelled.".to_string()));
    }
    let validation = validate_payload(content);
    if !matches!(validation, ApplyOutcome::Success { .. }) {
        return Ok(validation);
    }
    apply_to_stage(content, ctx)
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

fn validate_payload(content: &str) -> ApplyOutcome {
    let manifest = match manifest::parse_manifest(content) {
        Ok(Some(m)) => m,
        Ok(None) => Vec::new(),
        Err(e) => return ApplyOutcome::ParseError(format!("Manifest Error: {e}")),
    };
    let extracted = match extractor::extract_files(content) {
        Ok(e) => e,
        Err(e) => return ApplyOutcome::ParseError(format!("Extraction Error: {e}")),
    };
    validator::validate(&manifest, &extracted)
}

fn apply_to_stage(content: &str, ctx: &ApplyContext) -> Result<ApplyOutcome> {
    let extracted = extractor::extract_files(content)?;
    let manifest = manifest::parse_manifest(content)?.unwrap_or_default();

    if ctx.dry_run {
        return Ok(ApplyOutcome::Success {
            written: vec!["(Dry Run) Verified".to_string()],
            deleted: vec![],
            backed_up: false,
            staged: false,
        });
    }

    let (mut stage, outcome) = execute_stage_transaction(&manifest, &extracted, ctx)?;

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
        println!("{}", "� Verification passed!".green().bold());
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

    // NOTE: We do NOT print here. The ApplyOutcome::Promoted will be handled
    // by messages::print_outcome at the top level to avoid double printing.

    Ok(ApplyOutcome::Promoted {
        written: result.files_written,
        deleted: result.files_deleted,
    })
}

fn print_stage_info(stage: &StageManager) {
    println!(
        "\n{} Changes staged. Run {} to verify, or {} to promote.",
        "".blue(),
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

/// Promotes staged changes to the real workspace.
///
/// # Errors
/// Returns error if promotion fails.
pub fn run_promote(ctx: &ApplyContext) -> Result<ApplyOutcome> {
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