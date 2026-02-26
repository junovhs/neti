use crate::reporting::guidance::get_guidance;
use crate::reporting::shared::{
    collect_violations, confidence_suffix, duration, next_occurrence, pluralize, rule_counts,
};
use crate::types::{ScanReport, Violation};
use anyhow::Result;
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::path::Path;

/// Formats a report as a string (for embedding in context files).
///
/// # Errors
/// Returns error if formatting fails.
pub fn format_report_string(report: &ScanReport) -> Result<String> {
    let mut out = String::new();

    for file in report.files.iter().filter(|f| !f.is_clean()) {
        for v in &file.violations {
            writeln!(
                out,
                "FILE: {} | LAW: {} | {}: {} | LINE: {} | {}",
                file.path.display(),
                v.law,
                v.confidence.prefix().to_uppercase(),
                v.confidence.label(),
                v.row,
                v.message
            )?;
        }
    }

    Ok(out)
}

/// Builds a rich, multi-line report string without ANSI colors for file logging.
/// This matches the exact fidelity of the console output.
///
/// # Errors
/// Returns error if formatting fails.
pub fn build_rich_report(report: &ScanReport) -> Result<String> {
    let mut out = String::new();

    let all = collect_violations(report);
    let counts = rule_counts(&all);

    let mut shown: HashMap<&'static str, usize> = HashMap::new();

    for (path, v) in &all {
        let total = counts.get(v.law).copied().unwrap_or(1);
        let occurrence = next_occurrence(&mut shown, v.law);

        if occurrence == 1 {
            write_violation_full(&mut out, path, v, occurrence, total)?;
        } else {
            write_violation_compact(&mut out, path, v, occurrence, total)?;
        }
    }

    write_summary(&mut out, report)?;
    Ok(out)
}

fn write_violation_full(
    out: &mut String,
    path: &Path,
    v: &Violation,
    occurrence: usize,
    total: usize,
) -> Result<()> {
    let count_label = if total > 1 {
        format!(" [{occurrence} of {total}]")
    } else {
        String::new()
    };

    writeln!(out, "{}:{count_label} {}", v.confidence.prefix(), v.message)?;
    writeln!(out, "  --> {}:{}", path.display(), v.row)?;

    write_snippet(out, path, v.row)?;

    writeln!(out, "   = {}: {}", v.law, confidence_suffix(v))?;

    if let Some(ref details) = v.details {
        if !details.analysis.is_empty() {
            writeln!(out, "   |")?;
            writeln!(out, "   = ANALYSIS:")?;
            for line in &details.analysis {
                writeln!(out, "   |   {line}")?;
            }
        }
    }

    if let Some(guidance) = get_guidance(v.law) {
        writeln!(out, "   |")?;
        writeln!(out, "   = WHY: {}", guidance.why)?;
        writeln!(out, "   |")?;
        writeln!(out, "   = FIX: {}", guidance.fix)?;
    }

    writeln!(out, "   |")?;
    writeln!(
        out,
        "   = SUPPRESS: // neti:allow({}) on the line, or {} = \"warn\" in neti.toml [rules]",
        v.law, v.law
    )?;
    writeln!(out)?;
    Ok(())
}

fn write_violation_compact(
    out: &mut String,
    path: &Path,
    v: &Violation,
    occurrence: usize,
    total: usize,
) -> Result<()> {
    writeln!(
        out,
        "{}: [{occurrence} of {total}] {}",
        v.confidence.prefix(),
        v.message
    )?;
    writeln!(out, "  --> {}:{}", path.display(), v.row)?;

    if let Some(ref details) = v.details {
        if !details.analysis.is_empty() {
            let brief = details.analysis.first().map_or("", String::as_str);
            writeln!(out, "   = {}: {} — see first {} above", v.law, brief, v.law)?;
        } else {
            writeln!(out, "   = {}: see first {} above", v.law, v.law)?;
        }
    } else {
        writeln!(out, "   = {}: see first {} above", v.law, v.law)?;
    }

    writeln!(out)?;
    Ok(())
}

fn write_snippet(out: &mut String, path: &Path, row: usize) -> Result<()> {
    let Ok(content) = fs::read_to_string(path) else {
        return Ok(());
    };
    let lines: Vec<&str> = content.lines().collect();

    let idx = row.saturating_sub(1);
    let start = idx.saturating_sub(1);
    let end = (idx + 1).min(lines.len().saturating_sub(1));

    writeln!(out, "   |")?;

    for i in start..=end {
        if let Some(line) = lines.get(i) {
            let line_num = i + 1;
            let gutter = format!("{line_num:3} |");

            if i == idx {
                writeln!(out, "   {} {}", gutter, line)?;
                let trimmed = line.trim_start();
                let padding = line.len() - trimmed.len();
                let underline_len = trimmed.len().max(1);
                let spaces = " ".repeat(padding);
                let carets = "^".repeat(underline_len);
                writeln!(out, "   | {}{}", spaces, carets)?;
            } else {
                writeln!(out, "   {} {}", gutter, line)?;
            }
        }
    }

    Ok(())
}

fn write_summary(out: &mut String, report: &ScanReport) -> Result<()> {
    let duration = duration(report);

    let errors = report.error_count();
    let warnings = report.warning_count();
    let suggestions = report.suggestion_count();

    if errors == 0 && warnings == 0 && suggestions == 0 {
        writeln!(out, "OK No violations found in {duration:?}.")?;
        return Ok(());
    }

    let mut parts: Vec<String> = Vec::new();
    if errors > 0 {
        parts.push(format!("{} {}", errors, pluralize("error", errors)));
    }
    if warnings > 0 {
        parts.push(format!("{} {}", warnings, pluralize("warning", warnings)));
    }
    if suggestions > 0 {
        parts.push(format!(
            "{} {}",
            suggestions,
            pluralize("suggestion", suggestions)
        ));
    }

    let summary = parts.join(", ");

    if errors > 0 {
        writeln!(out, "X Neti found {summary} ({duration:?}).")?;
    } else {
        writeln!(out, "~ Neti found {summary} ({duration:?}).")?;
    }

    Ok(())
}
