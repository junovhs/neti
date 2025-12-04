// src/apply/messages.rs
use crate::apply::types::ApplyOutcome;
use colored::Colorize;

pub fn print_outcome(outcome: &ApplyOutcome) {
    match outcome {
        ApplyOutcome::Success {
            written,
            deleted,
            roadmap_results,
            backed_up,
        } => print_success(written, deleted, roadmap_results, *backed_up),
        ApplyOutcome::ValidationFailure {
            errors,
            missing,
            ai_message,
        } => {
            print_validation_errors(errors, missing);
            print_ai_feedback(ai_message);
        }
        ApplyOutcome::ParseError(e) => println!("{}: {e}", "⚠️  Parse Error".red()),
        ApplyOutcome::WriteError(e) => println!("{}: {e}", "💥 Write Error".red()),
    }
}

fn print_success(written: &[String], deleted: &[String], roadmap: &[String], backed_up: bool) {
    println!("{}", "✅ Apply successful!".green().bold());
    if backed_up {
        println!("   (Backup created in .slopchop_apply_backup/)");
    }
    println!();

    for file in written {
        println!("   {} {file}", "✓".green());
    }
    for file in deleted {
        println!("   {} {file}", "✗".red());
    }

    if !roadmap.is_empty() {
        println!("{}", "\n   Roadmap Updates:".cyan());
        for msg in roadmap {
            println!("   {msg}");
        }
    }

    println!();
    println!("Run {} to verify.", "slopchop check".yellow());
}

fn print_validation_errors(errors: &[String], missing: &[String]) {
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
}

pub fn print_ai_feedback(ai_message: &str) {
    println!();
    println!("{}", "📋 Paste this back to the AI:".cyan().bold());
    println!("{}", "─".repeat(60).black());
    println!("{ai_message}");
    println!("{}", "─".repeat(60).black());

    if crate::clipboard::copy_to_clipboard(ai_message).is_ok() {
        println!("{}", "✓ Copied to clipboard".green());
    }
}

#[must_use]
pub fn format_ai_rejection(missing: &[String], errors: &[String]) -> String {
    use std::fmt::Write;
    let mut msg = String::from("The previous output was rejected by the SlopChop Protocol.\n\n");

    if !missing.is_empty() {
        msg.push_str(
            "MISSING FILES (Declared in MANIFEST but no #__SLOPCHOP_FILE__# block found):\n",
        );
        for f in missing {
            let _ = writeln!(msg, "- {f}");
        }
        msg.push('\n');
    }

    if !errors.is_empty() {
        msg.push_str("VALIDATION ERRORS:\n");
        let mut hint_dogfood = false;
        for e in errors {
            let _ = writeln!(msg, "- {e}");
            if e.contains("truncation marker") || e.contains("Banned") {
                hint_dogfood = true;
            }
        }
        msg.push('\n');

        if hint_dogfood {
            msg.push_str("TIP: If you are actively 'dogfooding' or intentionally using banned patterns, use '// slopchop:ignore' to bypass.\n\n");
        }
    }

    msg.push_str(
        "Please provide the missing or corrected files using #__SLOPCHOP_FILE__# path ... #__SLOPCHOP_END__#",
    );
    msg
}

#[must_use]
pub fn format_verification_failure(output: &str) -> String {
    format!(
        "The changes were applied, but post-application verification failed.\n\nFAILURE LOG:\n{}\n\nPlease fix the implementation so that checks pass.",
        output.trim()
    )
}
