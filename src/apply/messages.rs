// src/apply/messages.rs
use crate::apply::types::ApplyOutcome;
use colored::Colorize;

pub fn print_outcome(outcome: &ApplyOutcome) {
    match outcome {
        ApplyOutcome::Success {
            written,
            deleted,
            backed_up,
            staged,
        } => {
            if *staged {
                println!("{}", "✓ Staged successfully!".green().bold());
            } else {
                println!("{}", "✓ Apply successful!".green().bold());
            }
            if *backed_up {
                println!("   (Backup created)");
            }
            for file in written {
                println!("   {} {file}", "→".green());
            }
            for file in deleted {
                println!("   {} {file}", "✗".red());
            }
            if *staged {
                println!("\nRun {} to verify.", "slopchop check".yellow());
            }
        }
        ApplyOutcome::Promoted { written, deleted } => {
            println!("{}", "✓ Promoted to workspace!".green().bold());
            for file in written {
                println!("   {} {file}", "→".green());
            }
            for file in deleted {
                println!("   {} {file}", "✗".red());
            }
        }
        ApplyOutcome::StageReset => {
            println!("{}", "✓ Stage reset.".green());
        }
        ApplyOutcome::ValidationFailure { ai_message, .. } => {
            println!("{}", "✗ Validation Failed".red().bold());
            print_ai_feedback(ai_message);
        }
        ApplyOutcome::ParseError(e) => {
            println!("{}: {e}", "⚠ Parse Error".red());
        }
        ApplyOutcome::WriteError(e) => {
            println!("{}: {e}", "⚠ Write Error".red());
        }
    }
}

pub fn print_ai_feedback(ai_message: &str) {
    println!("\n{}", "→ Paste this back to the AI:".cyan().bold());
    println!("{ai_message}");
    let _ = crate::clipboard::copy_to_clipboard(ai_message);
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
