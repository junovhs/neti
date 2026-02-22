//! Console output formatting for scan results.
//!
//! Violations are grouped and deduplicated by rule code. The first occurrence
//! of each rule shows the full educational block (analysis, why, fix, suppress).
//! Subsequent occurrences of the same rule show a compact one-liner with a
//! back-reference.

use crate::types::{Confidence, ScanReport, Violation};
use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::time::Duration;

/// Static educational guidance per rule.
struct RuleGuidance {
    why: &'static str,
    fix: &'static str,
}

/// Returns educational guidance for a rule code, if available.
fn get_guidance(rule: &str) -> Option<RuleGuidance> {
    Some(match rule {
        "P01" => RuleGuidance {
            why: "Cloning/copying inside a loop allocates on every iteration, scaling linearly with iteration count.",
            fix: "Hoist the allocation before the loop, use a reference or borrow, or confirm the copy is cheap (primitives, small structs, reference-counted pointers).",
        },
        "P02" => RuleGuidance {
            why: "String conversion inside a loop allocates a new String on every iteration.",
            fix: "Hoist the conversion before the loop, or operate on borrowed string slices (&str).",
        },
        "P04" => RuleGuidance {
            why: "Nested loops produce O(n²) complexity, which scales poorly with input size.",
            fix: "Replace the inner loop with a lookup structure (HashMap/HashSet) for O(n) total, or confirm the inner loop is bounded to a small constant.",
        },
        "P06" => RuleGuidance {
            why: "Linear search (.find/.position/.index) inside a loop produces O(n·m) complexity.",
            fix: "Pre-build a lookup structure (HashSet/HashMap/dict/Set) for O(1) access, or confirm the inner collection is bounded to a small constant size.",
        },
        "L02" => RuleGuidance {
            why: "Using <= or >= with .len() in index bounds can reach len, which is one past the last valid index.",
            fix: "Use < len for upper bounds on indices. The valid index range is 0..len-1.",
        },
        "L03" => RuleGuidance {
            why: "Indexing without a bounds proof panics on empty or undersized collections at runtime.",
            fix: "Use safe accessors (.first()/.get()), add an emptiness guard, or prove the collection size is guaranteed by construction (fixed-size array, chunks_exact).",
        },
        "X01" => RuleGuidance {
            why: "Building SQL from string formatting allows injection when inputs are user-controlled.",
            fix: "Use parameterized queries (? placeholders) with your database driver's bind API.",
        },
        "X02" => RuleGuidance {
            why: "Executing external commands with dynamic arguments risks injection (shell) or untrusted binary resolution (direct exec).",
            fix: "For shell commands: validate and sanitize inputs, or avoid shell invocation entirely. For direct exec: use absolute paths, allowlists, or signature verification.",
        },
        "C03" => RuleGuidance {
            why: "Holding a lock guard across an await point blocks the executor thread (sync mutex) or starves other tasks (async mutex).",
            fix: "Scope the guard so it drops before the await, or extract the critical section into a synchronous helper function.",
        },
        "C04" => RuleGuidance {
            why: "Synchronization primitives without documentation make concurrent code harder to reason about and audit.",
            fix: "Add a comment explaining what the lock protects and the expected contention pattern.",
        },
        "I01" => RuleGuidance {
            why: "Manual From implementations are boilerplate that can be generated with derive macros.",
            fix: "Use derive_more::From if your project already depends on proc macros. Manual impls are perfectly fine for zero-dependency crates.",
        },
        "I02" => RuleGuidance {
            why: "Duplicate match arm bodies indicate arms that could be combined with the | pattern.",
            fix: "Combine arms: `A | B => shared_body`. Only valid when bindings have compatible types.",
        },
        "M03" | "M04" | "M05" => RuleGuidance {
            why: "Function name implies a contract (getter, predicate, pure computation) that the implementation violates.",
            fix: "Rename the function to match its behavior, or refactor the implementation to match its name.",
        },
        "R07" => RuleGuidance {
            why: "Buffered writers that are dropped without flushing may silently lose data.",
            fix: "Call .flush() explicitly before the writer goes out of scope, or return it so the caller controls lifetime.",
        },
        "S01" | "S02" | "S03" => RuleGuidance {
            why: "Global mutable state creates hidden coupling and makes code harder to test and reason about.",
            fix: "Pass state explicitly via function parameters, or use dependency injection patterns.",
        },
        "LAW OF PARANOIA" => RuleGuidance {
            why: "Unsafe blocks must document their safety invariants so reviewers can verify correctness.",
            fix: "Add a // SAFETY: comment immediately above the unsafe block explaining why the invariants hold.",
        },
        "LAW OF ATOMICITY" => RuleGuidance {
            why: "Files beyond the token limit are too large for a single unit of work, increasing cognitive load and merge conflict risk.",
            fix: "Split the file into smaller, focused modules. Extract related functions into their own files.",
        },
        "LAW OF INTEGRITY" => RuleGuidance {
            why: "Syntax errors prevent analysis and indicate malformed or unparseable code.",
            fix: "Fix the syntax error, or if this is valid modern syntax that Neti's parser doesn't support, file an issue.",
        },
        _ => return None,
    })
}

/// Prints a formatted scan report to stdout with confidence tiers and deduplication.
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

/// Collects all violations with their file paths, then prints them grouped by rule
/// with deduplication: first occurrence gets full educational detail, subsequent
/// occurrences get a compact back-reference.
fn print_violations_grouped(report: &ScanReport) {
    let mut all: Vec<(&Path, &Violation)> = Vec::new();
    for file in &report.files {
        for v in &file.violations {
            all.push((&file.path, v));
        }
    }

    let mut rule_counts: HashMap<&str, usize> = HashMap::new();
    for (_, v) in &all {
        *rule_counts.entry(v.law).or_insert(0) += 1;
    }

    let mut rule_shown: HashMap<&str, usize> = HashMap::new();

    for (path, v) in &all {
        let total = rule_counts.get(v.law).copied().unwrap_or(1);
        let occurrence = {
            let entry = rule_shown.entry(v.law).or_insert(0);
            *entry += 1;
            *entry
        };

        if occurrence == 1 {
            print_violation_full(path, v, occurrence, total);
        } else {
            print_violation_compact(path, v, occurrence, total);
        }
    }
}

fn print_violation_full(path: &Path, v: &Violation, occurrence: usize, total: usize) {
    let path_str = path.display().to_string();
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

    println!("  {} {}:{}", "-->".blue(), path_str, v.row);
    print_snippet(path, v.row);

    let confidence_suffix = match v.confidence {
        Confidence::High => v.confidence.label().to_string(),
        Confidence::Medium => {
            let reason = v
                .confidence_reason
                .as_deref()
                .unwrap_or("pattern match without proof");
            format!("{} — {reason}", v.confidence.label())
        }
        Confidence::Info => v.confidence.label().to_string(),
    };
    println!(
        "   {} {}: {}",
        "=".blue(),
        v.law.yellow(),
        confidence_suffix
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
    let path_str = path.display().to_string();
    let prefix = v.confidence.prefix();

    let header = format!("{prefix}: [{occurrence} of {total}] {}", v.message);
    match v.confidence {
        Confidence::High => println!("{}", header.red().bold()),
        Confidence::Medium => println!("{}", header.yellow()),
        Confidence::Info => println!("{}", header.dimmed()),
    }

    println!("  {} {}:{}", "-->".blue(), path_str, v.row);

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
    #[allow(clippy::cast_possible_truncation)]
    let duration = Duration::from_millis(report.duration_ms as u64);

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

fn pluralize(word: &str, count: usize) -> String {
    if count == 1 {
        word.to_string()
    } else {
        format!("{word}s")
    }
}

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

/// Prints a serializable object as JSON to stdout.
///
/// # Errors
/// Returns error if serialization fails.
pub fn print_json<T: serde::Serialize>(data: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    println!("{json}");
    Ok(())
}
