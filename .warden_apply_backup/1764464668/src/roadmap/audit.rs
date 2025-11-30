// src/roadmap/audit.rs
use crate::roadmap::slugify;
use crate::roadmap::types::{Roadmap, Task, TaskStatus};
use colored::Colorize;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

pub struct AuditOptions {
    pub strict: bool,
}

pub fn run(roadmap: &Roadmap, root: &Path, _opts: AuditOptions) {
    println!("{}", "🕵️  Roadmap Traceability Audit".bold().cyan());
    println!("{}", "─────────────────────────────────────".dimmed());

    let tasks = roadmap.all_tasks();
    let completed: Vec<&&Task> = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Complete)
        .collect();

    if completed.is_empty() {
        println!("{}", "No completed tasks to audit.".yellow());
        return;
    }

    let test_files = find_test_files(root);
    let mut missing_count = 0;

    for task in completed {
        if !has_test(task, &test_files) {
            print_missing(task);
            missing_count += 1;
        }
    }

    print_summary(missing_count);
}

fn print_missing(task: &Task) {
    println!(
        "{} {} (id: {})",
        "⚠️  Missing Test:".red(),
        task.text.bold(),
        task.id.dimmed()
    );
}

fn print_summary(missing: usize) {
    println!();
    if missing == 0 {
        println!("{}", "✅ All completed tasks have associated tests!".green().bold());
    } else {
        println!(
            "{}",
            format!("❌ Found {missing} tasks without detected tests.").red().bold()
        );
        println!("   (Tip: Add <!-- test: tests/my_test.rs --> to the task in ROADMAP.md)");
    }
}

fn find_test_files(root: &Path) -> Vec<String> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_ignored_dir(e))
        .flatten()
        .filter(is_test_file)
        .filter_map(|e| e.path().to_str().map(str::to_lowercase))
        .collect()
}

fn is_ignored_dir(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_str().unwrap_or("");
    name.starts_with('.') || name == "target" || name == "node_modules" || name == "vendor"
}

fn is_test_file(entry: &DirEntry) -> bool {
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
        .is_some_and(|ext| {
            ext.eq_ignore_ascii_case("rs")
                || ext.eq_ignore_ascii_case("ts")
                || ext.eq_ignore_ascii_case("js")
                || ext.eq_ignore_ascii_case("py")
                || ext.eq_ignore_ascii_case("go")
        })
}

fn has_test(task: &Task, test_files: &[String]) -> bool {
    // 1. Check for explicit anchor
    if !task.tests.is_empty() {
        return task.tests.iter().all(|t| {
             let target = t.to_lowercase();
             test_files.iter().any(|f| f.contains(&target))
        });
    }

    // 2. Fallback to slug heuristic
    let slug = slugify(&task.text).replace('-', "_");
    let id_slug = task.id.replace('-', "_");

    test_files
        .iter()
        .any(|f| f.contains(&slug) || f.contains(&id_slug))
}