// src/apply/report_writer.rs
//! Logic for generating the verification report file.
//! Extracted from verification.rs to satisfy Law of Atomicity.

use crate::types::{CommandResult, FileReport, ScanReport};
use anyhow::Result;
use std::fmt::Write;
use std::path::Path;

/// Writes the check report to a file.
///
/// # Errors
/// Returns error if file writing fails.
pub fn write_check_report(scan: &ScanReport, cmds: &[CommandResult], passed: bool, root: &Path) -> Result<()> {
    let mut out = String::with_capacity(10000);
    
    write_header(&mut out, passed)?;
    write_dashboard(&mut out, &scan.files)?;
    write_violations(&mut out, scan, cmds, passed)?;
    write_full_logs(&mut out, cmds)?;

    std::fs::write(root.join("slopchop-report.txt"), out)?;
    Ok(())
}

fn write_header(out: &mut String, passed: bool) -> Result<()> {
    writeln!(out, "SLOPCHOP CHECK REPORT")?;
    writeln!(out, "Generated: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"))?;
    writeln!(out, "Status: {}\n", if passed { "PASSED" } else { "FAILED" })?;
    Ok(())
}

fn write_dashboard(out: &mut String, files: &[FileReport]) -> Result<()> {
    writeln!(out, "=== DASHBOARD ===")?;
    let mut files_sorted = files.to_vec();
    
    writeln!(out, "Top 5 Cognitive Complexity:")?;
    files_sorted.sort_by(|a, b| b.complexity_score.cmp(&a.complexity_score));
    for f in files_sorted.iter().take(5) {
        writeln!(out, "  {:<4} {}", f.complexity_score, f.path.display())?;
    }
    
    writeln!(out, "\nTop 5 Largest Files (Tokens):")?;
    files_sorted.sort_by(|a, b| b.token_count.cmp(&a.token_count));
    for f in files_sorted.iter().take(5) {
        writeln!(out, "  {:<5} {}", f.token_count, f.path.display())?;
    }
    Ok(())
}

fn write_violations(out: &mut String, scan: &ScanReport, cmds: &[CommandResult], passed: bool) -> Result<()> {
    if passed {
        writeln!(out, "\n=== VIOLATIONS ===\nNone. Codebase is clean.")?;
        return Ok(());
    }

    writeln!(out, "\n=== VIOLATIONS ===")?;
    if scan.has_errors() {
        writeln!(out, "[SlopChop Internal Rules]")?;
        for file in scan.files.iter().filter(|f| !f.is_clean()) {
            for v in &file.violations {
                writeln!(out, "{}:{} | {} | {}", file.path.display(), v.row, v.law, v.message)?;
            }
        }
    }
    
    writeln!(out, "\n[External Tools]")?;
    for cmd in cmds {
        if cmd.exit_code != 0 {
            writeln!(out, "FAILED: {} (Exit Code: {})", cmd.command, cmd.exit_code)?;
            writeln!(out, "-- STDOUT --\n{}", cmd.stdout)?;
            writeln!(out, "-- STDERR --\n{}", cmd.stderr)?;
        }
    }
    Ok(())
}

fn write_full_logs(out: &mut String, cmds: &[CommandResult]) -> Result<()> {
    writeln!(out, "\n=== FULL OUTPUT LOGS ===")?;
    for cmd in cmds {
        writeln!(out, "\n>>> COMMAND: {}", cmd.command)?;
        if !cmd.stdout.is_empty() { writeln!(out, "{}", cmd.stdout)?; }
        if !cmd.stderr.is_empty() { writeln!(out, "{}", cmd.stderr)?; }
    }
    Ok(())
}