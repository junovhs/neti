// src/mutate/report.rs
//! Report formatting for mutation test results.


use crate::mutate::runner::{MutationResult, MutationSummary};
use colored::Colorize;
use std::fmt::Write;

/// Formats a progress update for terminal output.
#[must_use]
pub fn format_progress(current: usize, total: usize, result: &MutationResult) -> String {
    let status = if result.survived {
        "SURVIVED".yellow().bold()
    } else {
        "KILLED".green()
    };

    let file = result.point.file.display();
    let line = result.point.line;
    let orig = &result.point.original;
    let mutated = &result.point.mutated;

    format!(
        "[{current}/{total}] {file}:{line}  '{orig}' → '{mutated}'  ... {status}"
    )
}

/// Formats the final summary report.
#[must_use]
pub fn format_summary(summary: &MutationSummary) -> String {
    let mut out = String::new();

    let _ = writeln!(out);
    let _ = writeln!(out, "{}", "═".repeat(60));
    let _ = writeln!(out, "{}", "MUTATION TESTING COMPLETE".bold());
    let _ = writeln!(out, "{}", "═".repeat(60));
    let _ = writeln!(out);
    let _ = writeln!(out, "  Total mutations:  {}", summary.total);
    let _ = writeln!(out, "  Killed:           {} ✓", summary.killed.to_string().green());
    let _ = writeln!(out, "  Survived:         {} ⚠", format_survived(summary.survived));
    let _ = writeln!(out);

    let score_str = format!("{:.1}%", summary.score);
    let score_colored = if summary.score >= 80.0 {
        score_str.green().bold()
    } else if summary.score >= 60.0 {
        score_str.yellow().bold()
    } else {
        score_str.red().bold()
    };
    let _ = writeln!(out, "  Mutation Score:   {score_colored}");

    let duration_secs = summary.total_duration_ms / 1000;
    let _ = writeln!(out, "  Duration:         {duration_secs}s");
    let _ = writeln!(out);

    out
}

fn format_survived(count: usize) -> colored::ColoredString {
    if count == 0 {
        count.to_string().green()
    } else {
        count.to_string().yellow()
    }
}

/// Formats the surviving mutants report.
#[must_use]
pub fn format_survivors(results: &[MutationResult]) -> String {
    let survivors: Vec<_> = results.iter().filter(|r| r.survived).collect();

    if survivors.is_empty() {
        return format!("{}\n", "All mutants killed! Tests are solid.".green().bold());
    }

    let mut out = String::new();
    let _ = writeln!(out, "{}", "SURVIVING MUTANTS (test gaps)".yellow().bold());
    let _ = writeln!(out, "{}", "─".repeat(60));

    for result in &survivors {
        let kind = result.point.kind.symbol();
        let file = result.point.file.display();
        let line = result.point.line;
        let orig = &result.point.original;
        let mutated = &result.point.mutated;

        let _ = writeln!(
            out,
            "  [{}] {}:{}  '{}' → '{}'",
            kind.cyan(),
            file,
            line,
            orig.red(),
            mutated.green()
        );
    }

    let _ = writeln!(out);
    let len = survivors.len();
    let _ = writeln!(
        out,
        "{}: {len} mutations not caught by tests",
        "Action needed".yellow().bold()
    );

    out
}

/// Formats results as JSON for machine consumption.
#[must_use]
pub fn format_json(results: &[MutationResult], summary: &MutationSummary) -> String {
    let survivors: Vec<_> = results
        .iter()
        .filter(|r| r.survived)
        .map(|r| {
            serde_json::json!({
                "file": r.point.file.display().to_string(),
                "line": r.point.line,
                "column": r.point.column,
                "original": r.point.original,
                "mutated": r.point.mutated,
                "kind": r.point.kind.symbol(),
            })
        })
        .collect();

    let output = serde_json::json!({
        "summary": {
            "total": summary.total,
            "killed": summary.killed,
            "survived": summary.survived,
            "score": summary.score,
            "duration_ms": summary.total_duration_ms,
        },
        "survivors": survivors,
    });

    serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
}
