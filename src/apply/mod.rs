// src/apply/mod.rs
pub mod extractor;
pub mod git;
pub mod manifest;
pub mod messages;
pub mod types;
pub mod validator;
pub mod verification;
pub mod writer;

use crate::clipboard;
use crate::roadmap_v2;
use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, Write};
use std::path::Path;
use types::{ApplyContext, ApplyOutcome, ExtractedFiles, Manifest};

const INTENT_FILE: &str = ".slopchop_intent";

/// Runs the apply command logic.
///
/// # Errors
/// Returns error if clipboard access fails or git state is invalid.
pub fn run_apply(ctx: &ApplyContext) -> Result<ApplyOutcome> {
    check_git_state(ctx)?;
    let content = clipboard::read_clipboard().context("Failed to read clipboard")?;
    process_input(&content, ctx)
}

fn check_git_state(ctx: &ApplyContext) -> Result<()> {
    if ctx.dry_run || ctx.force || !git::in_repo() || ctx.config.preferences.allow_dirty_git {
        return Ok(());
    }

    if git::is_dirty()? {
        println!("{}", "[WARN] Git working tree has uncommitted changes.".yellow());
    }
    Ok(())
}

pub fn print_result(outcome: &ApplyOutcome) {
    messages::print_outcome(outcome);
}

/// Processes input content directly.
///
/// # Errors
/// Returns error if extraction, write, or git operations fail.
pub fn process_input(content: &str, ctx: &ApplyContext) -> Result<ApplyOutcome> {
    if content.trim().is_empty() {
        return Ok(ApplyOutcome::ParseError(
            "Clipboard/Input is empty".to_string(),
        ));
    }

    let plan_opt = extractor::extract_plan(content);

    if !check_plan_requirement(plan_opt.as_deref(), ctx)? {
        return Ok(ApplyOutcome::ParseError(
            "Operation cancelled: Plan missing or rejected.".to_string(),
        ));
    }

    let validation = validate_payload(content);
    if !matches!(validation, ApplyOutcome::Success { .. }) {
        return Ok(validation);
    }

    apply_and_verify(content, ctx, plan_opt.as_deref())
}

fn check_plan_requirement(plan: Option<&str>, ctx: &ApplyContext) -> Result<bool> {
    if let Some(p) = plan {
        println!("{}", "[PLAN]:".cyan().bold());
        println!("{}", "-".repeat(50).dimmed());
        println!("{}", p.trim());
        println!("{}", "-".repeat(50).dimmed());
        
        if !ctx.force && !ctx.dry_run {
            return confirm("Apply these changes?");
        }
        return Ok(true);
    }

    if ctx.config.preferences.require_plan {
        println!("{}", "[X] REJECTED: Input missing PLAN block.".red());
        Ok(false)
    } else {
        println!("{}", "[WARN] No PLAN block found.".yellow());
        if !ctx.force && !ctx.dry_run {
            return confirm("Apply without a plan?");
        }
        Ok(true)
    }
}

fn validate_payload(content: &str) -> ApplyOutcome {
    let manifest = match parse_manifest_step(content) {
        Ok(m) => m,
        Err(e) => return ApplyOutcome::ParseError(e),
    };

    let extracted = match extract_files_step(content) {
        Ok(e) => e,
        Err(e) => return ApplyOutcome::ParseError(e),
    };

    validator::validate(&manifest, &extracted)
}

fn apply_and_verify(content: &str, ctx: &ApplyContext, plan: Option<&str>) -> Result<ApplyOutcome> {
    let extracted = extractor::extract_files(content)?;
    let manifest = manifest::parse_manifest(content)?.unwrap_or_default();

    if ctx.dry_run {
        return Ok(ApplyOutcome::Success {
            written: vec!["(Dry Run) Files verified".to_string()],
            deleted: vec![],
            roadmap_results: vec![],
            backed_up: false,
        });
    }

    let mut outcome = writer::write_files(&manifest, &extracted, None)?;
    let roadmap_path = Path::new("tasks.toml");
    let mut roadmap_results = Vec::new();

    match roadmap_v2::handle_input(roadmap_path, content) {
        Ok(results) => roadmap_results = results,
        Err(e) => {
            if content.contains("===ROADMAP===") {
                eprintln!("{} Roadmap update failed: {e}", "[WARN]".yellow());
            }
        }
    }

    if let ApplyOutcome::Success {
        roadmap_results: ref mut rr,
        ..
    } = outcome
    {
        rr.append(&mut roadmap_results);
    }

    verify_and_commit(&outcome, ctx, plan)?;
    Ok(outcome)
}

fn verify_and_commit(outcome: &ApplyOutcome, ctx: &ApplyContext, plan: Option<&str>) -> Result<()> {
    if !matches!(outcome, ApplyOutcome::Success { .. }) || !has_changes(outcome) {
        if !has_changes(outcome) {
            println!("{}", "No changes detected.".yellow());
        }
        return Ok(());
    }

    // New unified pipeline: Lint -> Test -> Scan
    let success = verification::run_verification_pipeline(ctx)?;

    if success {
        handle_success(plan);
    } else {
        println!(
            "{}",
            "\n[X] Verification Failed. Changes applied but NOT committed."
                .red()
                .bold()
        );
        if let Some(p) = plan {
            save_intent(p);
        }
    }
    Ok(())
}

fn has_changes(outcome: &ApplyOutcome) -> bool {
    if let ApplyOutcome::Success {
        written,
        deleted,
        roadmap_results,
        ..
    } = outcome
    {
        !written.is_empty() || !deleted.is_empty() || !roadmap_results.is_empty()
    } else {
        false
    }
}

fn handle_success(plan: Option<&str>) {
    println!(
        "{}",
        "\n[OK] Verification Passed. Committing & Pushing..."
            .green()
            .bold()
    );
    let message = construct_commit_message(plan);
    if let Err(e) = git::commit_and_push(&message) {
        eprintln!("{} Git operation failed: {e}", "[WARN]".yellow());
    } else {
        clear_intent();
    }
}

fn save_intent(plan: &str) {
    if !Path::new(INTENT_FILE).exists() {
        let clean = plan.replace("GOAL:", "").trim().to_string();
        let _ = std::fs::write(INTENT_FILE, clean);
    }
}

fn clear_intent() {
    let _ = std::fs::remove_file(INTENT_FILE);
}

fn construct_commit_message(current_plan: Option<&str>) -> String {
    let current = current_plan
        .unwrap_or("Automated update")
        .replace("GOAL:", "")
        .trim()
        .to_string();

    if let Ok(stored) = std::fs::read_to_string(INTENT_FILE) {
        let stored = stored.trim();
        if !stored.is_empty() && stored != current {
            return format!("{stored}\n\nFollow-up: {current}");
        }
    }
    current
}

fn confirm(prompt: &str) -> Result<bool> {
    print!("{prompt} [y/N] ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}

fn parse_manifest_step(content: &str) -> Result<Manifest, String> {
    match manifest::parse_manifest(content) {
        Ok(Some(m)) => Ok(m),
        Ok(None) => Ok(Vec::new()),
        Err(e) => Err(format!("Manifest Error: {e}")),
    }
}

fn extract_files_step(content: &str) -> Result<ExtractedFiles, String> {
    extractor::extract_files(content).map_err(|e| format!("Extraction Error: {e}"))
}