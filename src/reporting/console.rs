use crate::reporting::guidance::get_guidance;
use crate::reporting::shared::{
    collect_violations, confidence_suffix, duration, next_occurrence, pluralize, rule_counts,
};
use crate::types::{Confidence, ScanReport, Violation};
use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Prints a formatted scan report to stdout with confidence tiers and
/// deduplication.
///
/// # Errors
/// Returns error if formatting fails.
pub fn print_report(report: &ScanReport) -> Result<()> {
    if report.has_errors() {
        print_violations_grouped(report);
    }
    print_summary(report);
    Ok(())
}

/// Collects all violations with their file paths, then prints them grouped by
/// rule with deduplication: first occurrence gets full educational detail,
/// subsequent occurrences get a compact back-reference.
fn print_violations_grouped(report: &ScanReport) {
    let all = collect_violations(report);
    let counts = rule_counts(&all);

    let mut shown: HashMap<&'static str, usize> = HashMap::new();

    for (path, v) in &all {
        let total = counts.get(v.law).copied().unwrap_or(1);
        let occurrence = next_occurrence(&mut shown, v.law);

        if occurrence == 1 {
            print_violation_full(path, v, occurrence, total);
        } else {
            print_violation_compact(path, v, occurrence, total);
        }
    }
}

fn print_violation_full(path: &Path, v: &Violation, occurrence: usize, total: usize) {
    let prefix = v.confidence.prefix();
    let count_label = if total > 1 {
        format!(" [{occurrence} of {total}]")
    } else {
        String::new()
    };

    let header = format!("{prefix}:{count_label} {}", v.message);
    match v.confidence {
        Confidence::High => println!("{}", header.red().bold()),
        Confidence::Medium => println!("{}", header.yellow()),
        Confidence::Info => println!("{}", header.dimmed()),
    }

    println!("  {} {}:{}", "-->".blue(), path.display(), v.row);
    print_snippet(path, v.row);

    println!(
        "   {} {}: {}",
        "=".blue(),
        v.law.yellow(),
        confidence_suffix(v)
    );

    if let Some(ref details) = v.details {
        if !details.analysis.is_empty() {
            println!("   {}", "|".blue());
            println!("   {} {}", "=".blue(), "ANALYSIS:".cyan());
            for line in &details.analysis {
                println!("   {}   {}", "|".blue(), line.dimmed());
            }
        }
    }

    if let Some(guidance) = get_guidance(v.law) {
        println!("   {}", "|".blue());
        println!("   {} {} {}", "=".blue(), "WHY:".cyan(), guidance.why);
        println!("   {}", "|".blue());
        println!("   {} {} {}", "=".blue(), "FIX:".green(), guidance.fix);
    }

    println!("   {}", "|".blue());
    println!(
        "   {} {} {}",
        "=".blue(),
        "SUPPRESS:".dimmed(),
        format!(
            "// neti:allow({}) on the line, or {} = \"warn\" in neti.toml [rules]",
            v.law, v.law
        )
        .dimmed()
    );

    println!();
}

fn print_violation_compact(path: &Path, v: &Violation, occurrence: usize, total: usize) {
    let prefix = v.confidence.prefix();
    let header = format!("{prefix}: [{occurrence} of {total}] {}", v.message);

    match v.confidence {
        Confidence::High => println!("{}", header.red().bold()),
        Confidence::Medium => println!("{}", header.yellow()),
        Confidence::Info => println!("{}", header.dimmed()),
    }

    println!("  {} {}:{}", "-->".blue(), path.display(), v.row);

    if let Some(ref details) = v.details {
        if !details.analysis.is_empty() {
            let brief = details.analysis.first().map_or("", String::as_str);
            println!(
                "   {} {}: {} — see first {} above",
                "=".blue(),
                v.law.yellow(),
                brief.dimmed(),
                v.law
            );
        } else {
            println!(
                "   {} {}: see first {} above",
                "=".blue(),
                v.law.yellow(),
                v.law
            );
        }
    } else {
        println!(
            "   {} {}: see first {} above",
            "=".blue(),
            v.law.yellow(),
            v.law
        );
    }

    println!();
}

fn print_snippet(path: &Path, row: usize) {
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };
    let lines: Vec<&str> = content.lines().collect();

    let idx = row.saturating_sub(1);
    let start = idx.saturating_sub(1);
    let end = (idx + 1).min(lines.len().saturating_sub(1));

    println!("   {}", "|".blue());

    for i in start..=end {
        if let Some(line) = lines.get(i) {
            let line_num = i + 1;
            let gutter = format!("{line_num:3} |");

            if i == idx {
                println!("   {} {}", gutter.blue(), line);
                let trimmed = line.trim_start();
                let padding = line.len() - trimmed.len();
                let underline_len = trimmed.len().max(1);
                let spaces = " ".repeat(padding);
                let carets = "^".repeat(underline_len);
                println!("   {} {}{}", "|".blue(), spaces, carets.red().bold());
            } else {
                println!("   {} {}", gutter.blue().dimmed(), line.dimmed());
            }
        }
    }
}

fn print_summary(report: &ScanReport) {
    let duration = duration(report);

    let errors = report.error_count();
    let warnings = report.warning_count();
    let suggestions = report.suggestion_count();

    if errors == 0 && warnings == 0 && suggestions == 0 {
        println!(
            "{} No violations found in {duration:?}.",
            "OK".green().bold()
        );
        return;
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
        println!("{} Neti found {summary} ({duration:?}).", "X".red().bold());
    } else {
        println!(
            "{} Neti found {summary} ({duration:?}).",
            "~".yellow().bold()
        );
    }
}
