// src/roadmap_v2/cli/handlers.rs
use crate::clipboard;
use crate::roadmap_v2::parser::parse_commands;
use crate::roadmap_v2::types::{RoadmapMeta, Section, SectionStatus, TaskStatus, TaskStore};
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::io::{self, Read};
use std::path::Path;

use super::display;

pub fn run_init(output: &Path, name: Option<String>) -> Result<()> {
    if output.exists() {
        return Err(anyhow!(
            "{} already exists. Use --output.",
            output.display()
        ));
    }

    let title = name.unwrap_or_else(|| "Project".to_string());
    let store = create_template_store(&title);

    store.save(Some(output)).map_err(|e| anyhow!("{e}"))?;
    println!("{} Created {}", "✓".green(), output.display());
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
        if should_show_task(&task.status, pending, complete) {
            let mark = match task.status {
                TaskStatus::Done | TaskStatus::NoTest => "[x]",
                TaskStatus::Pending => "[ ]",
            };
            println!("{mark} {} - {}", task.id, task.text);
        }
    }
    Ok(())
}

fn should_show_task(status: &TaskStatus, pending: bool, complete: bool) -> bool {
    match (pending, complete) {
        (true, false) => *status == TaskStatus::Pending,
        (false, true) => *status == TaskStatus::Done || *status == TaskStatus::NoTest,
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

    if dry_run {
        display::print_dry_run(&commands);
        return Ok(());
    }

    let (success_count, errors) = apply_all_commands(&mut store, commands, verbose);

    if success_count > 0 {
        store.save(Some(file)).map_err(|e| anyhow!("{e}"))?;
        println!("{} Applied {success_count} command(s)", "✓".green());
    }

    for err in &errors {
        eprintln!("{} {err}", "✗".red());
    }

    Ok(())
}

fn apply_all_commands(
    store: &mut TaskStore,
    commands: Vec<crate::roadmap_v2::RoadmapCommand>,
    verbose: bool,
) -> (usize, Vec<String>) {
    let mut success_count = 0;
    let mut errors: Vec<String> = Vec::new();

    for cmd in commands {
        if verbose {
            println!("  Applying: {cmd:?}");
        }
        match store.apply(cmd) {
            Ok(()) => success_count += 1,
            Err(e) => errors.push(format!("{e}")),
        }
    }

    (success_count, errors)
}

pub fn run_generate(source: &Path, output: &Path) -> Result<()> {
    let store = load_store(source)?;
    let markdown = store.to_markdown();

    std::fs::write(output, markdown)?;
    println!("{} Generated {}", "✓".green(), output.display());
    Ok(())
}

pub fn run_audit(file: &Path, strict: bool) -> Result<()> {
    let store = load_store(file)?;
    let root = std::env::current_dir()?;

    display::print_audit_header();

    let failures = count_audit_failures(&store, &root);

    display::print_audit_result(failures, strict)
}

fn count_audit_failures(store: &TaskStore, root: &Path) -> usize {
    let mut failures = 0;

    for task in &store.tasks {
        // Only audit completed tasks - skip pending and no-test
        if task.status != TaskStatus::Done {
            continue;
        }

        if let Some(fail) = check_task_test(task, root) {
            display::print_audit_failure(&task.text, &task.id, fail);
            failures += 1;
        }
    }

    failures
}

fn check_task_test(task: &crate::roadmap_v2::Task, root: &Path) -> Option<&'static str> {
    match &task.test {
        Some(t) if t == "[no-test]" => None,
        Some(test_path) if !verify_test_exists(root, test_path) => Some("test not found"),
        None => Some("no test anchor"),
        Some(_) => None,
    }
}

fn verify_test_exists(root: &Path, test_path: &str) -> bool {
    let parts: Vec<&str> = test_path.split("::").collect();
    let file_path = root.join(parts.first().unwrap_or(&""));

    if !file_path.exists() {
        return false;
    }

    if parts.len() > 1 {
        let fn_name = parts[1];
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            return content.contains(&format!("fn {fn_name}"));
        }
    }

    true
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
