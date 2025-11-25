// src/apply/messages.rs
use crate::apply::types::ApplyOutcome;
use colored::Colorize;

pub fn print_outcome(outcome: &ApplyOutcome) {
    match outcome {
        ApplyOutcome::Success { written, backed_up } => {
            println!("{}", "✅ Apply successful!".green().bold());
            if *backed_up {
                println!("   (Backup created in .warden_apply_backup/)");
            }
            println!();
            for file in written {
                println!("   {} {file}", "✓".green());
            }
            println!();
            println!("Run {} to verify.", "warden check".yellow());
        }
        ApplyOutcome::ValidationFailure {
            errors,
            missing,
            ai_message,
        } => {
            println!("{}", "❌ Validation Failed".red().bold());

            if !missing.is_empty() {
                println!(
                    "{}",
                    "\nMissing Files (Declared but not provided):".yellow()
                );
                for f in missing {
                    println!("   - {f}");
                }
            }

            if !errors.is_empty() {
                println!("{}", "\nContent Errors:".yellow());
                for e in errors {
                    println!("   - {e}");
                }
            }

            println!();
            println!("{}", "📋 Paste this back to the AI:".cyan().bold());
            println!("{}", "─".repeat(60).black());
            println!("{ai_message}");
            println!("{}", "─".repeat(60).black());

            if crate::clipboard::copy_to_clipboard(ai_message).is_ok() {
                println!("{}", "✓ Copied to clipboard".green());
            }
        }
        ApplyOutcome::ParseError(e) => {
            println!("{}: {e}", "⚠️  Parse Error".red());
        }
        ApplyOutcome::WriteError(e) => {
            println!("{}: {e}", "💥 Write Error".red());
        }
    }
}

#[must_use]
pub fn format_ai_rejection(missing: &[String], errors: &[String]) -> String {
    use std::fmt::Write;
    let mut msg = String::from("The previous output was rejected by the Warden Protocol.\n\n");

    if !missing.is_empty() {
        msg.push_str("MISSING FILES (Declared in <delivery> but not found in <file> blocks):\n");
        for f in missing {
            let _ = writeln!(msg, "- {f}");
        }
        msg.push('\n');
    }

    if !errors.is_empty() {
        msg.push_str("VALIDATION ERRORS:\n");
        for e in errors {
            let _ = writeln!(msg, "- {e}");
        }
        msg.push('\n');
    }

    msg.push_str(
        "Please provide the missing or corrected files using the <file path=\"...\"> format.",
    );
    msg
}
