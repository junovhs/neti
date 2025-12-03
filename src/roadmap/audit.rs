// src/roadmap/audit.rs
use crate::roadmap::slugify;
use crate::roadmap::types::{Roadmap, Task, TaskStatus};
use colored::Colorize;
use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Clone, Copy)]
pub struct AuditOptions {
    pub strict: bool,
}

#[derive(Debug)]
pub struct AuditViolation {
    pub task_id: String,
    pub task_text: String,
    pub reason: ViolationReason,
}

#[derive(Debug)]
pub enum ViolationReason {
    MissingTestFile(String),
    MissingTestFunction { file: String, function: String },
    NoTraceability, // Heuristic failed
}

#[derive(Debug)]
pub struct AuditReport {
    pub violations: Vec<AuditViolation>,
    pub total_checked: usize,
}

impl AuditReport {
    fn new() -> Self {
        Self {
            violations: Vec::new(),
            total_checked: 0,
        }
    }
}

#[must_use]
pub fn run(roadmap: &Roadmap, root: &Path, opts: AuditOptions) -> bool {
    println!("{}", "🕵️  Roadmap Traceability Audit".bold().cyan());
    println!("{}", "─────────────────────────────────────".dimmed());

    let report = scan(roadmap, root, &opts);

    if report.total_checked == 0 {
        println!("{}", "No completed tasks to audit.".yellow());
        return true;
    }

    for violation in &report.violations {
        print_violation(violation);
    }

    print_summary(report.violations.len());
    report.violations.is_empty()
}

#[must_use]
pub fn scan(roadmap: &Roadmap, root: &Path, _opts: &AuditOptions) -> AuditReport {
    let tasks = roadmap.all_tasks();
    let completed: Vec<&&Task> = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Complete)
        .collect();

    if completed.is_empty() {
        return AuditReport::new();
    }

    // Heuristic scan for un-anchored tasks
    let scanned_test_files = scan_test_files(root);
    let mut report = AuditReport::new();
    report.total_checked = completed.len();

    for task in completed {
        // Skip if marked as [no-test]
        if task.text.contains("[no-test]") {
            continue;
        }

        if let Some(reason) = check_task(task, root, &scanned_test_files) {
            report.violations.push(AuditViolation {
                task_id: task.id.clone(),
                task_text: task.text.clone(),
                reason,
            });
        }
    }

    report
}

fn check_task(task: &Task, root: &Path, scanned_files: &[String]) -> Option<ViolationReason> {
    // 1. Priority: Explicit Anchors
    if !task.tests.is_empty() {
        for test_ref in &task.tests {
            if let Some(reason) = verify_anchor(test_ref, root) {
                return Some(reason);
            }
        }
        return None;
    }

    // 2. Fallback: Slug Heuristic
    let slug = slugify(&task.text).replace('-', "_");
    let id_slug = task.id.replace('-', "_");

    let found = scanned_files
        .iter()
        .any(|f| f.contains(&slug) || f.contains(&id_slug));

    if found {
        None
    } else {
        Some(ViolationReason::NoTraceability)
    }
}

fn verify_anchor(anchor: &str, root: &Path) -> Option<ViolationReason> {
    // Support "path/to/file.rs::function_name" syntax
    let (file_part, fn_part) = if let Some((f, n)) = anchor.split_once("::") {
        (f, Some(n))
    } else {
        (anchor, None)
    };

    let path = root.join(file_part.trim());
    
    if !path.exists() || !path.is_file() {
        return Some(ViolationReason::MissingTestFile(file_part.trim().to_string()));
    }

    // If function name is specified, verify it exists in the file content
    if let Some(func_name) = fn_part {
        let name = func_name.trim();
        if let Ok(content) = fs::read_to_string(&path) {
            if !check_definition(&path, &content, name) {
                return Some(ViolationReason::MissingTestFunction {
                    file: file_part.trim().to_string(),
                    function: name.to_string(),
                });
            }
        }
    }

    None
}

fn check_definition(path: &Path, content: &str, name: &str) -> bool {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let pattern = build_definition_pattern(ext, name);

    let Ok(re) = Regex::new(&pattern) else {
        return content.contains(name);
    };

    // Iterate matches and check if line is commented
    for m in re.find_iter(content) {
        if !is_match_commented(content, m.start(), ext) {
            return true;
        }
    }
    
    false
}

fn build_definition_pattern(ext: &str, name: &str) -> String {
    match ext {
        "rs" => format!(r"fn\s+{name}\b"),
        "py" => format!(r"def\s+{name}\b"),
        "go" => format!(r"func\s+{name}\b"),
        "js" | "ts" | "jsx" | "tsx" => {
            // JS/TS is flexible: function foo, const foo =, foo: function
            format!(r"(function\s+{name}\b|const\s+{name}\s*=|let\s+{name}\s*=|var\s+{name}\s*=|{name}\s*[:\(])")
        }
        _ => name.to_string(), // Fallback (used as regex pattern if simple)
    }
}

fn is_match_commented(content: &str, start_idx: usize, ext: &str) -> bool {
    let line_start = content[..start_idx].rfind('\n').map_or(0, |i| i + 1);
    let prefix = content[line_start..start_idx].trim();
    
    match ext {
        "py" => prefix.starts_with('#'),
        _ => prefix.starts_with("//") || prefix.starts_with('*'),
    }
}

fn print_violation(v: &AuditViolation) {
    let msg = match &v.reason {
        ViolationReason::MissingTestFile(f) => format!("Missing File: {f}"),
        ViolationReason::MissingTestFunction { file, function } => {
            format!("Missing Function: '{function}' in {file}")
        }
        ViolationReason::NoTraceability => "No test file found (heuristic)".to_string(),
    };

    println!(
        "{} {} (id: {})",
        "⚠️  Traceability Fail:".red(),
        v.task_text.bold(),
        v.task_id.dimmed()
    );
    println!("   └─ {msg}");
}

fn print_summary(missing: usize) {
    println!();
    if missing == 0 {
        println!("{}", "✅ All completed tasks have verified tests!".green().bold());
    } else {
        println!(
            "{}",
            format!("❌ Found {missing} tasks without verified tests.").red().bold()
        );
        println!("   (Tip: Add <!-- test: tests/my_test.rs::function_name --> to the task in ROADMAP.md)");
    }
}

fn scan_test_files(root: &Path) -> Vec<String> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_ignored_dir(e))
        .flatten()
        .filter(is_heuristic_match)
        .filter_map(|e| e.path().to_str().map(str::to_lowercase))
        .collect()
}

fn is_ignored_dir(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_str().unwrap_or("");
    name.starts_with('.') || name == "target" || name == "node_modules" || name == "vendor"
}

/// Strict filter for the heuristic scanner.
/// Only picks up files that explicitly look like tests.
fn is_heuristic_match(entry: &DirEntry) -> bool {
    if !entry.file_type().is_file() {
        return false;
    }
    
    if !has_code_extension(entry.path()) {
        return false;
    }

    let Some(name) = entry.file_name().to_str() else {
        return false;
    };

    name.contains("test")
        || name.contains("spec")
        || entry.path().components().any(|c| c.as_os_str() == "tests")
}

fn has_code_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "rs" | "ts" | "js" | "py" | "go"
            )
        })
}