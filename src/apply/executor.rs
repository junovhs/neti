// src/apply/executor.rs
//! Handles the execution of apply actions with automatic branch management.

use crate::apply::types::{ApplyContext, ApplyOutcome, ExtractedFiles, Manifest};
use crate::apply::verification;
use crate::apply::writer;
use crate::branch;
use crate::events::{EventKind, EventLogger};
use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};

/// Executes the full apply transaction with automatic branch management.
///
/// # Errors
/// Returns error if git operations or file writing fails.
pub fn apply_to_stage_transaction(
    manifest: &Manifest,
    extracted: &ExtractedFiles,
    ctx: &ApplyContext,
    commit_msg: &str,
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

    // Step 1: Ensure we're on work branch
    ensure_work_branch()?;

    // Step 2: Write files
    let retention = ctx.config.preferences.backup_retention;
    let outcome = writer::write_files(manifest, extracted, Some(&ctx.repo_root), retention)?;

    // Log outcome
    log_outcome(&logger, &outcome);

    // Step 3: Run verification if requested
    if ctx.check_after {
        return run_verification_and_maybe_promote(ctx, outcome, commit_msg);
    }

    // Even if no check requested, we still commit the changes to the work branch
    // to preserve the transaction.
    commit_work_branch_changes(commit_msg)?;
    print_work_branch_status();
    Ok(outcome)
}

/// Ensures we're on the work branch, creating it if needed.
fn ensure_work_branch() -> Result<()> {
    if branch::on_work_branch() {
        return Ok(());
    }

    match branch::init_branch(false) {
        Ok(branch::BranchResult::Created) => {
            println!("{}", "→ Created work branch 'slopchop-work'".blue());
        }
        Ok(branch::BranchResult::AlreadyOnBranch) => {}
        Ok(branch::BranchResult::Reset) => {
            println!("{}", "→ Reset work branch".blue());
        }
        Err(e) => {
            if e.to_string().contains("already exists") {
                switch_to_work_branch()?;
            } else {
                return Err(e);
            }
        }
    }
    Ok(())
}

fn switch_to_work_branch() -> Result<()> {
    use std::process::Command;
    let output = Command::new("git")
        .args(["checkout", branch::work_branch_name()])
        .output()?;

    if !output.status.success() {
        anyhow::bail!("Failed to switch to work branch");
    }
    println!("{}", "→ Switched to work branch 'slopchop-work'".blue());
    Ok(())
}

fn log_outcome(logger: &EventLogger, outcome: &ApplyOutcome) {
    match outcome {
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
}

fn run_verification_and_maybe_promote(
    ctx: &ApplyContext,
    outcome: ApplyOutcome,
    commit_msg: &str,
) -> Result<ApplyOutcome> {
    let result = verification::run_verification_pipeline(ctx, &ctx.repo_root)?;

    if result.passed {
        println!("{}", "✓ All checks passed!".green().bold());

        // Commit changes on work branch first
        commit_work_branch_changes(commit_msg)?;

        // Auto-promote or prompt
        if ctx.auto_promote {
            return promote_to_main();
        }

        if confirm("Promote to main?")? {
            return promote_to_main();
        }

        println!("{}", "Changes committed on 'slopchop-work'. Run 'slopchop promote' when ready.".cyan());
        Ok(outcome)
    } else {
        println!("{}", "✗ Verification failed. Changes are on work branch.".yellow());

        let modified_files: Vec<String> = match &outcome {
            ApplyOutcome::Success { written, .. } => written.clone(),
            _ => Vec::new(),
        };

        let ai_msg = crate::apply::messages::generate_ai_feedback(&result, &modified_files);
        crate::apply::messages::print_ai_feedback(&ai_msg);

        println!("\n{}", "Fix the issues and run 'slopchop apply' again, or 'slopchop abort' to abandon.".cyan());
        Ok(outcome)
    }
}

fn commit_work_branch_changes(msg: &str) -> Result<()> {
    use std::process::Command;

    // Stage all changes
    let add = Command::new("git")
        .args(["add", "-A"])
        .output()?;

    if !add.status.success() {
        anyhow::bail!("Failed to stage changes");
    }

    // Check if there's anything to commit
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;

    if status.stdout.is_empty() {
        return Ok(()); // Nothing to commit
    }

    // Commit
    let commit = Command::new("git")
        .args(["commit", "-m", msg])
        .output()?;

    if !commit.status.success() {
        let stderr = String::from_utf8_lossy(&commit.stderr);
        anyhow::bail!("Failed to commit: {stderr}");
    }

    Ok(())
}

fn promote_to_main() -> Result<ApplyOutcome> {
    match branch::promote(false)? {
        branch::PromoteResult::Merged => {
            println!("{}", "✓ Promoted to main. Work branch cleaned up.".green().bold());
            Ok(ApplyOutcome::Promoted {
                written: vec![],
                deleted: vec![],
            })
        }
        branch::PromoteResult::DryRun => {
            Ok(ApplyOutcome::Success {
                written: vec![],
                deleted: vec![],
                backed_up: false,
                staged: false,
            })
        }
    }
}

fn print_work_branch_status() {
    if branch::on_work_branch() {
        println!("\n{}", "Changes applied on work branch. Run 'slopchop check' to verify.".cyan());
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
pub fn run_promote_standalone(_ctx: &ApplyContext) -> Result<ApplyOutcome> {
    if !branch::on_work_branch() {
        println!("{}", "Not on work branch. Nothing to promote.".yellow());
        return Ok(ApplyOutcome::ParseError("Not on work branch.".to_string()));
    }

    promote_to_main()
}