// src/roadmap_v2/cli/migrate.rs
use crate::roadmap_v2::types::{Section, SectionStatus, Task, TaskStatus, TaskStore};
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::path::Path;

pub fn run_migrate(input: &Path, output: &Path) -> Result<()> {
    if output.exists() {
        return Err(anyhow!("{} already exists. Remove it first.", output.display()));
    }

    let content = std::fs::read_to_string(input)
        .context("Failed to read legacy ROADMAP.md")?;

    let store = parse_legacy_roadmap(&content);

    store.save(Some(output)).map_err(|e| anyhow!("{e}"))?;

    print_migration_result(&store, output);
    Ok(())
}

fn print_migration_result(store: &TaskStore, output: &Path) {
    println!("{} Migration complete!", "✓".green());
    println!("   Sections: {}", store.sections.len());
    println!("   Tasks:    {}", store.tasks.len());
    println!("   Output:   {}", output.display());
    println!();
    println!("{}", "Next steps:".yellow().bold());
    println!("1. Review {}", output.display());
    println!("2. Run: slopchop roadmap generate");
    println!("3. Verify ROADMAP.md looks correct");
}

fn parse_legacy_roadmap(content: &str) -> TaskStore {
    let mut store = TaskStore::default();
    let mut ctx = ParseContext::default();

    for line in content.lines() {
        parse_line(line.trim(), &mut store, &mut ctx);
    }

    store
}

#[derive(Default)]
struct ParseContext {
    current_section: Option<String>,
    current_group: Option<String>,
    section_order: u32,
    task_order: u32,
}

fn parse_line(trimmed: &str, store: &mut TaskStore, ctx: &mut ParseContext) {
    if let Some(title) = trimmed.strip_prefix("# ") {
        store.meta.title = title.to_string();
        return;
    }

    if let Some(heading) = trimmed.strip_prefix("## ") {
        add_section(store, ctx, heading);
        return;
    }

    if let Some(heading) = trimmed.strip_prefix("### ") {
        ctx.current_group = Some(heading.to_string());
        return;
    }

    if let Some(task) = parse_task_line(trimmed, ctx) {
        store.tasks.push(task);
        ctx.task_order += 1;
    }
}

fn add_section(store: &mut TaskStore, ctx: &mut ParseContext, heading: &str) {
    let section_id = slugify(heading);
    let status = detect_section_status(heading);
    store.sections.push(Section {
        id: section_id.clone(),
        title: heading.trim_end_matches(" ✓").to_string(),
        status,
        order: ctx.section_order,
    });
    ctx.current_section = Some(section_id);
    ctx.current_group = None;
    ctx.section_order += 1;
    ctx.task_order = 0;
}

fn parse_task_line(line: &str, ctx: &ParseContext) -> Option<Task> {
    let section_id = ctx.current_section.as_ref()?;
    let (base_status, rest) = extract_task_status(line)?;

    let rest = rest.trim();
    let (task_text, test_anchor) = extract_test_anchor(rest);
    let task_text = clean_task_text(&task_text);

    // Detect [no-test] marker and override status
    let status = detect_no_test_marker(&task_text, base_status);
    let task_text = strip_no_test_marker(&task_text);

    let id = slugify(&task_text);

    Some(Task {
        id,
        text: task_text,
        status,
        section: section_id.clone(),
        group: ctx.current_group.clone(),
        test: test_anchor,
        order: ctx.task_order,
    })
}

fn detect_no_test_marker(text: &str, base_status: TaskStatus) -> TaskStatus {
    if text.contains("[no-test]") {
        return TaskStatus::NoTest;
    }
    base_status
}

fn strip_no_test_marker(text: &str) -> String {
    text.replace("[no-test]", "")
        .replace("  ", " ")
        .trim()
        .to_string()
}

fn extract_task_status(line: &str) -> Option<(TaskStatus, &str)> {
    if let Some(rest) = line.strip_prefix("- [x]") {
        return Some((TaskStatus::Done, rest));
    }
    if let Some(rest) = line.strip_prefix("- [X]") {
        return Some((TaskStatus::Done, rest));
    }
    if let Some(rest) = line.strip_prefix("- [ ]") {
        return Some((TaskStatus::Pending, rest));
    }
    None
}

fn extract_test_anchor(input: &str) -> (String, Option<String>) {
    let Some(start) = input.find("<!-- test:") else {
        return (input.to_string(), None);
    };
    let Some(end) = input[start..].find("-->") else {
        return (input.to_string(), None);
    };

    let anchor = input[start + 10..start + end].trim().to_string();
    let clean = input[..start].trim().to_string();
    (clean, Some(anchor))
}

fn clean_task_text(input: &str) -> String {
    input.trim()
        .trim_start_matches("**")
        .trim_end_matches("**")
        .trim()
        .to_string()
}

fn detect_section_status(heading: &str) -> SectionStatus {
    if heading.contains('✓') {
        return SectionStatus::Complete;
    }
    if heading.contains("CURRENT") {
        return SectionStatus::Current;
    }
    SectionStatus::Pending
}

fn slugify(input: &str) -> String {
    input.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '.')
        .collect::<String>()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}