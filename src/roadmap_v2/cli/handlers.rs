// src/roadmap_v2/cli/handlers.rs
use crate::clipboard;
use crate::roadmap_v2::parser::parse_commands;
use crate::roadmap_v2::types::{RoadmapMeta, Section, SectionStatus, TaskStatus, TaskStore};
use crate::roadmap_v2::validation;
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::io::{self, Read};
use std::path::Path;

use super::display;
use super::verify::{self, VerifyResult};

pub fn run_init(output: &Path, name: Option<String>) -> Result<()> {
    if output.exists() {
        return Err(anyhow!("{} already exists.", output.display()));
    }

    let title = name.unwrap_or_else(|| "Project".to_string());
    let store = create_template_store(&title);

    store.save(Some(output)).map_err(|e| anyhow!("{e}"))?;
    println!("{} Created {}", "�".green(), output.display());
    Ok(())
}

fn create_template_store(title: &str) -> TaskStore {
    TaskStore {
        meta: RoadmapMeta {
            title: format!("{title} Roadmap"),
            description: String::new(),
        },
        sections: vec![
            Section {
                id: "v0.1.0".to_string(),
                title: "v0.1.0".to_string(),
                status: SectionStatus::Current,
                order: 0,
            },
            Section {
                id: "v0.2.0".to_string(),
                title: "v0.2.0".to_string(),
                status: SectionStatus::Pending,
                order: 1,
            },
        ],
        tasks: vec![],
    }
}

pub fn run_show(file: &Path, format: &str) -> Result<()> {
    let store = load_store(file)?;

    if format == "stats" {
        display::print_stats(&store);
    } else {
        display::print_tree(&store);
    }
    Ok(())
}

pub fn run_tasks(file: &Path, pending: bool, complete: bool) -> Result<()> {
    let store = load_store(file)?;

    for task in &store.tasks {
        if should_show_task(task.status, pending, complete) {
            let mark = match task.status {
                TaskStatus::Done | TaskStatus::NoTest => "[x]",
                TaskStatus::Pending => "[ ]",
            };
            println!("{mark} {} - {}", task.id, task.text);
        }
    }
    Ok(())
}

fn should_show_task(status: TaskStatus, pending: bool, complete: bool) -> bool {
    match (pending, complete) {
        (true, false) => status == TaskStatus::Pending,
        (false, true) => status == TaskStatus::Done || status == TaskStatus::NoTest,
        _ => true,
    }
}

pub fn run_apply(file: &Path, dry_run: bool, stdin: bool, verbose: bool) -> Result<()> {
    let mut store = load_store(file)?;
    let input = get_input(stdin)?;
    let commands = parse_commands(&input).map_err(|e| anyhow!("{e}"))?;

    if commands.is_empty() {
        return Err(anyhow!("No ===ROADMAP=== commands found."));
    }

    println!("Found {} command(s)", commands.len());

    // Pre-validation
    let report = validation::validate_batch(&store, &commands);
    print_validation_report(&report, verbose);

    if !report.is_ok() {
        return Err(anyhow!("Validation failed. Fix errors before applying."));
    }

    if dry_run {
        display::print_dry_run(&commands);
        return Ok(());
    }

    let result = store.apply_batch(commands)?;

    if result.applied > 0 {
        store.save_with_backup(Some(file)).map_err(|e| anyhow!("{e}"))?;
        println!("{} Applied {} command(s)", "�".green(), result.applied);
    }

    for err in &result.errors {
        eprintln!("{} {err}", "?".red());
    }

    Ok(())
}

fn print_validation_report(report: &validation::ValidationReport, verbose: bool) {
    for err in &report.errors {
        eprintln!("{} {err}", "?".red());
    }

    if verbose {
        for warn in &report.warnings {
            eprintln!("{} {warn}", "?".yellow());
        }
    }
}

pub fn run_generate(source: &Path, output: &Path) -> Result<()> {
    let store = load_store(source)?;
    let markdown = store.to_markdown();

    std::fs::write(output, markdown)?;
    println!("{} Generated {}", "�".green(), output.display());
    Ok(())
}

pub fn run_audit(file: &Path, strict: bool, exec: bool) -> Result<()> {
    let store = load_store(file)?;
    let root = std::env::current_dir()?;

    display::print_audit_header();

    if exec {
        println!("  {} Running tests...\n", "?".yellow());
    }

    let failures = count_audit_failures(&store, &root, exec);

    display::print_audit_result(failures, strict)
}

fn count_audit_failures(store: &TaskStore, root: &Path, exec: bool) -> usize {
    let mut failures = 0;

    for task in &store.tasks {
        if task.status != TaskStatus::Done {
            continue;
        }

        let result = verify::check_test(&task.id, task.test.as_deref(), root, exec);
        if let Some(reason) = result_to_reason(result) {
            display::print_audit_failure(&task.text, &task.id, reason);
            failures += 1;
        }
    }

    failures
}

fn result_to_reason(result: VerifyResult) -> Option<&'static str> {
    match result {
        VerifyResult::Pass | VerifyResult::Skipped => None,
        VerifyResult::NotFound => Some("test not found"),
        VerifyResult::NoAnchor => Some("no test anchor"),
        VerifyResult::ExecFailed => Some("test failed"),
    }
}

pub fn load_store(path: &Path) -> Result<TaskStore> {
    TaskStore::load(Some(path)).map_err(|e| anyhow!("{e}"))
}

fn get_input(stdin: bool) -> Result<String> {
    if stdin {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        Ok(buf)
    } else {
        clipboard::read_clipboard().context("Clipboard read failed")
    }
}