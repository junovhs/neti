// src/roadmap_v2/cli/display.rs
use crate::roadmap_v2::types::{AddCommand, AfterTarget, SectionStatus, TaskStatus, TaskStore};
use crate::roadmap_v2::RoadmapCommand;
use anyhow::{anyhow, Result};
use colored::Colorize;

pub fn print_stats(store: &TaskStore) {
    let total = store.tasks.len();
    let done = store
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Done || t.status == TaskStatus::NoTest)
        .count();
    let pending = total - done;

    println!("?? Roadmap Stats");
    println!("   Total:   {total}");
    println!("   Done:    {done}");
    println!("   Pending: {pending}");

    if total > 0 {
        #[allow(clippy::cast_precision_loss)]
        let pct = (done as f64 / total as f64) * 100.0;
        println!("   Progress: {pct:.1}%");
    }
}

pub fn print_tree(store: &TaskStore) {
    println!("?? {}\n", store.meta.title);

    for section in &store.sections {
        print_section(store, section);
    }
}

fn print_section(store: &TaskStore, section: &crate::roadmap_v2::types::Section) {
    let marker = match section.status {
        SectionStatus::Complete => "�",
        SectionStatus::Current => "",
        SectionStatus::Pending => "	",
    };

    println!("{marker} {}", section.title.bold());

    let tasks: Vec<_> = store
        .tasks
        .iter()
        .filter(|t| t.section == section.id)
        .collect();

    for task in tasks {
        print_task(task);
    }
    println!();
}

fn print_task(task: &crate::roadmap_v2::types::Task) {
    let mark = match task.status {
        TaskStatus::Done => "[x]".green(),
        TaskStatus::NoTest => "[*]".yellow(),
        TaskStatus::Pending => "[ ]".dimmed(),
    };
    println!("  {mark} {}", task.text);
}

pub fn print_dry_run(commands: &[RoadmapCommand]) {
    println!("\n{}", "DRY RUN - No changes will be made:".yellow().bold());
    println!("{}", "�".repeat(40));

    for (i, cmd) in commands.iter().enumerate() {
        println!("{}. {}", i + 1, format_command(cmd));
    }

    println!("{}", "�".repeat(40));
}

fn format_command(cmd: &RoadmapCommand) -> String {
    match cmd {
        RoadmapCommand::Check { id } => format!("{} {id}", "CHECK".green()),
        RoadmapCommand::Uncheck { id } => format!("{} {id}", "UNCHECK".yellow()),
        RoadmapCommand::Add(add) => format_add_command(add),
        RoadmapCommand::Update { id, fields } => {
            let changes: Vec<&str> = [
                fields.text.as_ref().map(|_| "text"),
                fields.test.as_ref().map(|_| "test"),
                fields.section.as_ref().map(|_| "section"),
            ]
            .into_iter()
            .flatten()
            .collect();
            format!("{} {id} ({})", "UPDATE".blue(), changes.join(", "))
        }
        RoadmapCommand::Delete { id } => format!("{} {id}", "DELETE".red()),
    }
}

fn format_add_command(add: &AddCommand) -> String {
    let after_str = match &add.after {
        AfterTarget::End => String::new(),
        AfterTarget::Previous => " [after: PREVIOUS]".dimmed().to_string(),
        AfterTarget::Id(id) => format!(" [after: {id}]").dimmed().to_string(),
        AfterTarget::Text(t) => format!(" [after: TEXT \"{t}\"]").dimmed().to_string(),
        AfterTarget::Line(n) => format!(" [at line: {n}]").dimmed().to_string(),
    };

    format!(
        "{} {}  {} \"{}\"{}",
        "ADD".cyan(),
        add.task.id,
        add.task.section,
        add.task.text,
        after_str
    )
}

pub fn print_audit_header() {
    println!("\n{}", "?? Roadmap Audit".bold());
    println!("{}\n", "�".repeat(40));
}

pub fn print_audit_failure(text: &str, id: &str, reason: &str) {
    println!("  {} {} ({})", "?".red(), text, id.dimmed());
    println!("    �� {}", reason.red());
}

pub fn print_audit_result(failures: usize, strict: bool) -> Result<()> {
    println!("{}", "�".repeat(40));

    if failures == 0 {
        println!("{} All tasks verified!", "�".green());
        return Ok(());
    }

    println!("{} {failures} task(s) failed verification", "?".yellow());

    if strict {
        return Err(anyhow!("Strict mode: failing due to verification errors"));
    }

    Ok(())
}