//! Prompt generation for AI interaction
//!
//! Generates clipboard-ready prompts that teach AI the roadmap command format.

use crate::parser::{Roadmap, TaskStatus};

/// Options for prompt generation
#[derive(Debug, Clone, Default)]
pub struct PromptOptions {
    /// Include full roadmap content vs compact summary
    pub full: bool,
    /// Include examples
    pub examples: bool,
    /// Project name override
    pub project_name: Option<String>,
}

/// Generate the teaching prompt for AI
pub fn generate_prompt(roadmap: &Roadmap, options: &PromptOptions) -> String {
    let project_name = options
        .project_name
        .clone()
        .unwrap_or_else(|| roadmap.title.clone());

    let mut prompt = String::new();

    // Header
    prompt.push_str(&format!("# Roadmap Commands for: {}\n\n", project_name));

    // Stats summary
    let stats = roadmap.stats();
    prompt.push_str(&format!(
        "Progress: {}/{} tasks complete ({:.0}%)\n\n",
        stats.complete,
        stats.total,
        if stats.total > 0 {
            stats.complete as f64 / stats.total as f64 * 100.0
        } else {
            0.0
        }
    ));

    // Command reference
    prompt.push_str("## Commands\n\n");
    prompt.push_str("Wrap commands in `===ROADMAP===` and `===END===` markers.\n\n");
    prompt.push_str("```\n");
    prompt.push_str("CHECK <path>              # Mark task complete\n");
    prompt.push_str("UNCHECK <path>            # Mark task incomplete\n");
    prompt.push_str("ADD <section> \"<text>\"    # Add new task to section\n");
    prompt.push_str("ADD <section> \"<text>\" AFTER <task>  # Add after specific task\n");
    prompt.push_str("DELETE <path>             # Remove task\n");
    prompt.push_str("UPDATE <path> \"<text>\"    # Change task description\n");
    prompt.push_str("NOTE <path> \"<text>\"      # Add note under task\n");
    prompt.push_str("```\n\n");

    // Path format explanation
    prompt.push_str("## Task Paths\n\n");
    prompt.push_str("Paths are: `section-slug/task-slug`\n");
    prompt.push_str("Example: `v0-5-0-bulletproof-apply/truncation-detection`\n\n");
    prompt.push_str("You can use partial matches - just the task slug often works.\n\n");

    // Examples
    if options.examples {
        prompt.push_str("## Examples\n\n");
        prompt.push_str("```\n");
        prompt.push_str("===ROADMAP===\n");
        prompt.push_str("CHECK truncation-detection\n");
        prompt.push_str("ADD v0-5-0 \"Improve error messages\" AFTER truncation-detection\n");
        prompt.push_str("NOTE path-safety \"Implemented using std::path::Path\"\n");
        prompt.push_str("===END===\n");
        prompt.push_str("```\n\n");
    }

    // Current state
    prompt.push_str("---\n\n");
    prompt.push_str("## Current Roadmap State\n\n");

    if options.full {
        // Include the raw roadmap
        prompt.push_str("```markdown\n");
        prompt.push_str(&roadmap.raw);
        prompt.push_str("\n```\n");
    } else {
        // Compact representation
        prompt.push_str(&generate_compact_state(roadmap));
    }

    prompt
}

/// Generate a minimal representation of current state
fn generate_compact_state(roadmap: &Roadmap) -> String {
    let mut out = String::new();

    for section in &roadmap.sections {
        if section.tasks.is_empty() && section.subsections.is_empty() {
            // Skip sections without tasks (like Philosophy, Principles)
            continue;
        }

        out.push_str(&format_section_tree(section, 0));
    }

    out
}

fn format_section_tree(section: &crate::parser::Section, depth: usize) -> String {
    let mut out = String::new();
    let indent = "  ".repeat(depth);

    // Count stats
    let (complete, total) = count_tasks_recursive(section);

    if total > 0 {
        let progress = format!("[{}/{}]", complete, total);
        let status_icon = if complete == total {
            "✓"
        } else if complete > 0 {
            "◐"
        } else {
            "○"
        };

        out.push_str(&format!(
            "{}{} {} {}\n",
            indent, status_icon, section.heading, progress
        ));

        // Show tasks with their IDs
        for task in &section.tasks {
            let icon = match task.status {
                TaskStatus::Complete => "  ✓",
                TaskStatus::Pending => "  ○",
            };
            out.push_str(&format!(
                "{}{}  {} (id: {})\n",
                indent, icon, task.text, task.id
            ));
        }
    }

    // Recurse into subsections
    for sub in &section.subsections {
        out.push_str(&format_section_tree(sub, depth + 1));
    }

    out
}

fn count_tasks_recursive(section: &crate::parser::Section) -> (usize, usize) {
    let mut complete = 0;
    let mut total = 0;

    for task in &section.tasks {
        total += 1;
        if task.status == TaskStatus::Complete {
            complete += 1;
        }
    }

    for sub in &section.subsections {
        let (c, t) = count_tasks_recursive(sub);
        complete += c;
        total += t;
    }

    (complete, total)
}

/// Generate a short status line for quick display
pub fn generate_status_line(roadmap: &Roadmap) -> String {
    let stats = roadmap.stats();
    let pct = if stats.total > 0 {
        stats.complete as f64 / stats.total as f64 * 100.0
    } else {
        0.0
    };

    format!(
        "{}: {}/{} ({:.0}%) complete",
        roadmap.title, stats.complete, stats.total, pct
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_prompt() {
        let content = r#"# Test Roadmap

## v1.0

- [x] Done task
- [ ] Pending task
"#;
        let roadmap = Roadmap::parse(content);
        let prompt = generate_prompt(&roadmap, &PromptOptions::default());

        assert!(prompt.contains("Test Roadmap"));
        assert!(prompt.contains("CHECK"));
        assert!(prompt.contains("1/2"));
    }
}
