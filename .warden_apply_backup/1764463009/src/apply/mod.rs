// src/apply/mod.rs
pub mod extractor;
pub mod git;
pub mod manifest;
pub mod messages;
pub mod types;
pub mod validator;
pub mod writer;

use crate::clipboard;
use crate::roadmap;
use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use types::{ApplyContext, ApplyOutcome, ExtractedFiles, Manifest};

const INTENT_FILE: &str = ".warden_intent";

/// Runs the apply command logic.
///
/// # Errors
/// Returns error if clipboard access fails.
pub fn run_apply(ctx: &ApplyContext) -> Result<ApplyOutcome> {
    let content = clipboard::read_clipboard().context("Failed to read clipboard")?;
    process_input(&content, ctx)
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
        return Ok(ApplyOutcome::ParseError("Clipboard/Input is empty".to_string()));
    }

    let plan_opt = extractor::extract_plan(content);

    if !ensure_consent(plan_opt.as_deref(), ctx)? {
        return Ok(ApplyOutcome::ParseError("Operation cancelled by user.".to_string()));
    }

    let validation = validate_payload(content);
    if !matches!(validation, ApplyOutcome::Success { .. }) {
        // Validation failed immediately (bad format/safety)
        // We do NOT persist intent here because the user likely needs to reprompt entirely.
        return Ok(validation);
    }

    apply_and_verify(content, ctx, plan_opt.as_deref())
}

fn ensure_consent(plan: Option<&str>, ctx: &ApplyContext) -> Result<bool> {
    let Some(p) = plan else {
        if ctx.force || ctx.dry_run {
            return Ok(true);
        }
        println!("{}", "⚠️  No PLAN block found. Proceed with caution.".yellow());
        return confirm("Apply these changes without a plan?");
    };

    println!("{}", "📋 PROPOSED PLAN:".cyan().bold());
    println!("{}", "─".repeat(50).dimmed());
    println!("{}", p.trim());
    println!("{}", "─".repeat(50).dimmed());

    if ctx.force || ctx.dry_run {
        return Ok(true);
    }

    validate_plan_structure(p);
    confirm("Apply these changes?")
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

    // Handle roadmap updates
    let roadmap_path = Path::new("ROADMAP.md");
    let mut roadmap_results = Vec::new();
    if roadmap_path.exists() {
        match roadmap::handle_input(roadmap_path, content) {
            Ok(results) => roadmap_results = results,
            Err(e) => eprintln!("{} Roadmap update failed: {e}", "⚠️".yellow()),
        }
    }
    if let ApplyOutcome::Success { roadmap_results: ref mut rr, .. } = outcome {
        rr.append(&mut roadmap_results);
    }

    verify_and_commit(&outcome, ctx, plan)?;
    Ok(outcome)
}

fn verify_and_commit(outcome: &ApplyOutcome, ctx: &ApplyContext, plan: Option<&str>) -> Result<()> {
    if !matches!(outcome, ApplyOutcome::Success { .. }) {
        return Ok(());
    }
    
    if !has_changes(outcome) {
         println!("{}", "No changes detected.".yellow());
         return Ok(());
    }

    if verify_application(ctx)? {
        handle_success(plan);
    } else {
        handle_failure(plan);
    }
    Ok(())
}

fn has_changes(outcome: &ApplyOutcome) -> bool {
    if let ApplyOutcome::Success { written, deleted, roadmap_results, .. } = outcome {
        !written.is_empty() || !deleted.is_empty() || !roadmap_results.is_empty()
    } else {
        false
    }
}

fn handle_success(plan: Option<&str>) {
    println!("{}", "\n✨ Verification Passed. Committing & Pushing...".green().bold());
    let message = construct_commit_message(plan);
    if let Err(e) = git::commit_and_push(&message) {
        eprintln!("{} Git operation failed: {e}", "⚠️".yellow());
    } else {
        clear_intent();
    }
}

fn handle_failure(plan: Option<&str>) {
    println!("{}", "\n❌ Verification Failed. Changes applied but NOT committed.".red().bold());
    println!("Fix the issues manually and then commit.");
    if let Some(p) = plan {
         save_intent(p);
    }
}

fn save_intent(plan: &str) {
    // Only save if no intent exists (preserve the original goal)
    if !Path::new(INTENT_FILE).exists() {
        let clean = plan.replace("GOAL:", "").trim().to_string();
        // Ignore errors silently (best effort)
        let _ = std::fs::write(INTENT_FILE, clean);
    }
}

fn clear_intent() {
    let _ = std::fs::remove_file(INTENT_FILE);
}

fn construct_commit_message(current_plan: Option<&str>) -> String {
    let current = current_plan.unwrap_or("Automated update").replace("GOAL:", "").trim().to_string();
    
    if let Ok(stored) = std::fs::read_to_string(INTENT_FILE) {
        let stored = stored.trim();
        if !stored.is_empty() && stored != current {
            return format!("{stored}\n\nFollow-up: {current}");
        }
    }
    current
}

fn verify_application(ctx: &ApplyContext) -> Result<bool> {
    println!("{}", "\n🔍 Verifying changes...".blue().bold());

    if let Some(cmd) = ctx.config.commands.get("check") {
        if !run_check_command(cmd)? {
            return Ok(false);
        }
    }

    println!("Running structural scan...");
    let status = Command::new("warden").status()?;
    Ok(status.success())
}

fn run_check_command(cmd: &str) -> Result<bool> {
    println!("Running check: {}", cmd.dimmed());
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let Some((prog, args)) = parts.split_first() else { return Ok(true); };
    let status = Command::new(prog).args(args).status()?;
    Ok(status.success())
}

fn validate_plan_structure(plan: &str) {
    if !plan.contains("GOAL:") || !plan.contains("CHANGES:") {
        println!("{}", "⚠️  Plan is unstructured (missing GOAL/CHANGES).".yellow());
    }
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