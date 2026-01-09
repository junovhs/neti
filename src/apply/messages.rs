// src/apply/messages.rs
use crate::apply::types::ApplyOutcome;
use crate::config::Config;
use crate::types::CheckReport;
use colored::Colorize;
use std::fmt::Write;

pub fn print_outcome(outcome: &ApplyOutcome) {
    match outcome {
        ApplyOutcome::Success {
            written,
            deleted,
            backed_up,
            staged,
        } => handle_success(written, deleted, *backed_up, *staged),
        ApplyOutcome::Promoted { written, deleted } => handle_promoted(written, deleted),
        ApplyOutcome::StageReset => println!("{}", "✓ Stage reset.".green()),
        ApplyOutcome::ValidationFailure { ai_message, .. } => handle_failure(ai_message),
        ApplyOutcome::ParseError(e) => println!("{}: {e}", "✗ Parse Error".red()),
        ApplyOutcome::WriteError(e) => println!("{}: {e}", "✗ Write Error".red()),
    }
}

fn handle_success(written: &[String], deleted: &[String], backed_up: bool, staged: bool) {
    if staged {
        println!("{}", "✓ Staged successfully!".green().bold());
    } else {
        println!("{}", "✓ Apply successful!".green().bold());
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
    println!("{}", "✓ Promoted to workspace!".green().bold());
    print_changes(written, deleted);
}

fn handle_failure(ai_message: &str) {
    println!("{}", "✗ Validation Failed".red().bold());
    print_ai_feedback(ai_message);
}

fn print_changes(written: &[String], deleted: &[String]) {
    for file in written {
        println!("   {} {file}", "+".green());
    }
    for file in deleted {
        println!("   {} {file}", "-".red());
    }
}

pub fn print_ai_feedback(ai_message: &str) {
    let config = Config::load();

    println!("\n{}", "📋 Paste this back to the AI:".cyan().bold());
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

#[must_use]
pub fn generate_ai_feedback(report: &CheckReport, modified_files: &[String]) -> String {
    let mut msg = String::from("VERIFICATION FAILED\n\n");

    msg.push_str("The following checks failed:\n\n");

    for cmd in report.commands.iter().filter(|c| c.exit_code != 0) {
        let _ = writeln!(msg, "COMMAND: {}", cmd.command);
        msg.push_str("OUTPUT:\n");
        let combined = format!("{}\n{}", cmd.stdout, cmd.stderr);
        let truncated = if combined.len() > 1000 {
            let safe_end = floor_char_boundary(&combined, 1000);
            format!("{}...\n[truncated]", &combined[..safe_end])
        } else {
            combined
        };
        msg.push_str(&truncated);
        msg.push_str("\n\n");
    }

    if report.scan.has_errors() {
        msg.push_str("COMMAND: slopchop scan\nOUTPUT:\nInternal violations found (see scan report).\n\n");
    }

    if !modified_files.is_empty() {
        msg.push_str("FILES MODIFIED IN THIS APPLY:\n");
        for f in modified_files {
            let _ = writeln!(msg, "- {f}");
        }
    }

    msg.push_str("\nPlease fix the issues and provide corrected files.");
    msg
}

/// Finds the largest valid char boundary <= idx.
fn floor_char_boundary(s: &str, mut idx: usize) -> usize {
    if idx >= s.len() {
        return s.len();
    }
    while !s.is_char_boundary(idx) {
        idx = idx.saturating_sub(1);
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CheckReport, CommandResult};

    #[test]
    fn test_floor_char_boundary() {
        let s = "abc🦀"; // bytes: 61 62 63 f0 9f 96 80
        
        // Exact boundaries
        assert_eq!(floor_char_boundary(s, 0), 0);
        assert_eq!(floor_char_boundary(s, 1), 1);
        assert_eq!(floor_char_boundary(s, 3), 3);
        
        // Inside crab (bytes 4, 5, 6) - should rewind to 3
        assert_eq!(floor_char_boundary(s, 4), 3);
        assert_eq!(floor_char_boundary(s, 5), 3);
        assert_eq!(floor_char_boundary(s, 6), 3);
        
        // End
        assert_eq!(floor_char_boundary(s, 7), 7);
        assert_eq!(floor_char_boundary(s, 100), 7);
    }

    #[test]
    fn test_feedback_truncation_utf8() {
        let prefix = "a".repeat(999);
        let s = format!("{prefix}🦀");
        
        let report = CheckReport {
            scan: crate::types::ScanReport::default(),
            commands: vec![CommandResult {
                command: "test".into(),
                exit_code: 1,
                stdout: s,
                stderr: String::new(),
                duration_ms: 0,
            }],
            passed: false,
        };
        
        let feedback = generate_ai_feedback(&report, &[]);
        
        assert!(feedback.contains("[truncated]"));
        assert!(feedback.contains(&prefix));
        assert!(!feedback.contains("🦀"));
    }
}