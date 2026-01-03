// src/apply/messages.rs
use crate::apply::types::ApplyOutcome;
use crate::config::Config;
use colored::Colorize;

pub fn print_outcome(outcome: &ApplyOutcome) {
    match outcome {
        ApplyOutcome::Success {
            written,
            deleted,
            backed_up,
            staged,
        } => handle_success(written, deleted, *backed_up, *staged),
        ApplyOutcome::Promoted { written, deleted } => handle_promoted(written, deleted),
        ApplyOutcome::StageReset => println!("{}", "� Stage reset.".green()),
        ApplyOutcome::ValidationFailure { ai_message, .. } => handle_failure(ai_message),
        ApplyOutcome::ParseError(e) => println!("{}: {e}", "? Parse Error".red()),
        ApplyOutcome::WriteError(e) => println!("{}: {e}", "? Write Error".red()),
    }
}

fn handle_success(written: &[String], deleted: &[String], backed_up: bool, staged: bool) {
    if staged {
        println!("{}", "� Staged successfully!".green().bold());
    } else {
        println!("{}", "� Apply successful!".green().bold());
    }
    if backed_up {
        println!("   (Backup created)");
    }
    print_changes(written, deleted);
    if staged {
        println!("\nRun {} to verify.", "slopchop check".yellow());
    }
}

fn handle_promoted(written: &[String], deleted: &[String]) {
    println!("{}", "� Promoted to workspace!".green().bold());
    print_changes(written, deleted);
}

fn handle_failure(ai_message: &str) {
    println!("{}", "? Validation Failed".red().bold());
    print_ai_feedback(ai_message);
}

fn print_changes(written: &[String], deleted: &[String]) {
    for file in written {
        println!("   {} {file}", " ".green());
    }
    for file in deleted {
        println!("   {} {file}", "?".red());
    }
}

pub fn print_ai_feedback(ai_message: &str) {
    let config = Config::load();

    println!("\n{}", "? Paste this back to the AI:".cyan().bold());
    println!("{ai_message}");

    if config.preferences.write_fix_packet {
        let path = &config.preferences.fix_packet_path;
        if let Err(e) = std::fs::write(path, ai_message) {
            eprintln!("Could not write fix packet to {path}: {e}");
        } else {
            println!("{}", format!("Fix packet written to: {path}").dimmed());
        }
    }

    if config.preferences.auto_copy {
        if let Err(e) = crate::clipboard::copy_to_clipboard(ai_message) {
            eprintln!("{}", format!("Warning: Auto-copy failed: {e}").yellow());
        } else {
            println!("{}", "  (Copied to clipboard)".dimmed());
        }
    }
}

#[must_use]
pub fn format_ai_rejection(missing: &[String], errors: &[String]) -> String {
    use std::fmt::Write;
    let mut msg = String::from("The previous output was rejected by the XSC7XSC Protocol.\n\n");
    if !missing.is_empty() {
        msg.push_str("MISSING FILES:\n");
        for f in missing {
            let _ = writeln!(msg, "- {f}");
        }
    }
    if !errors.is_empty() {
        msg.push_str("VALIDATION ERRORS:\n");
        for e in errors {
            let _ = writeln!(msg, "- {e}");
        }
    }
    msg.push_str("\nPlease provide the corrected files using XSC7XSC FILE XSC7XSC <path> ... XSC7XSC END XSC7XSC");
    msg
}