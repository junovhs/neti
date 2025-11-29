//! Parser for markdown roadmaps
//!
//! Extracts structure from markdown files while preserving the original format.

use std::collections::HashMap;
use std::path::Path;

/// A parsed roadmap document
#[derive(Debug, Clone)]
pub struct Roadmap {
    /// Original file path
    pub path: Option<String>,
    /// Document title (first H1)
    pub title: String,
    /// Top-level sections (Philosophy, Current State, versions, etc.)
    pub sections: Vec<Section>,
    /// Raw content for sections we don't parse deeply
    pub raw: String,
}

/// A section of the roadmap (version milestone, or meta-section)
#[derive(Debug, Clone)]
pub struct Section {
    /// Section ID (slugified from heading)
    pub id: String,
    /// Original heading text
    pub heading: String,
    /// Heading level (2 for ##, 3 for ###, etc.)
    pub level: u8,
    /// Optional theme/subtitle
    pub theme: Option<String>,
    /// Tasks in this section
    pub tasks: Vec<Task>,
    /// Subsections
    pub subsections: Vec<Section>,
    /// Raw markdown content (for non-task content)
    pub raw_content: String,
    /// Line number where this section starts
    pub line_start: usize,
    /// Line number where this section ends
    pub line_end: usize,
}

/// A single task item
#[derive(Debug, Clone)]
pub struct Task {
    /// Stable ID (slugified from text, or explicit <!-- id: xxx -->)
    pub id: String,
    /// Full path ID (section.subsection.task)
    pub path: String,
    /// Task text (without checkbox)
    pub text: String,
    /// Current status
    pub status: TaskStatus,
    /// Indentation level (0 = top-level, 1 = nested, etc.)
    pub indent: u8,
    /// Line number in original file
    pub line: usize,
    /// Child tasks (for nested lists)
    pub children: Vec<Task>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Complete,
}

impl Roadmap {
    /// Parse a roadmap from markdown content
    pub fn parse(content: &str) -> Self {
        let lines: Vec<&str> = content.lines().collect();
        let mut sections = Vec::new();
        let mut title = String::new();
        let mut i = 0;

        // Find title (first H1)
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("# ") && !line.starts_with("## ") {
                title = line[2..].trim().to_string();
                i += 1;
                break;
            }
            i += 1;
        }

        // Parse sections
        while i < lines.len() {
            if let Some((level, heading)) = parse_heading(lines[i]) {
                let section = parse_section(&lines, &mut i, level, heading);
                sections.push(section);
            } else {
                i += 1;
            }
        }

        Roadmap {
            path: None,
            title,
            sections,
            raw: content.to_string(),
        }
    }

    /// Parse from a file
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut roadmap = Self::parse(&content);
        roadmap.path = Some(path.display().to_string());
        Ok(roadmap)
    }

    /// Save back to file
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        std::fs::write(path, &self.raw)
    }

    /// Get a flat list of all tasks with their full paths
    pub fn all_tasks(&self) -> Vec<&Task> {
        let mut tasks = Vec::new();
        for section in &self.sections {
            collect_tasks(section, &mut tasks);
        }
        tasks
    }

    /// Find a task by its path (e.g., "v0.5.0/truncation-detection")
    pub fn find_task(&self, path: &str) -> Option<&Task> {
        self.all_tasks().into_iter().find(|t| t.path == path)
    }

    /// Find a task mutably by path
    pub fn find_task_mut(&mut self, path: &str) -> Option<&mut Task> {
        for section in &mut self.sections {
            if let Some(task) = find_task_mut_in_section(section, path) {
                return Some(task);
            }
        }
        None
    }

    /// Get summary statistics
    pub fn stats(&self) -> RoadmapStats {
        let tasks = self.all_tasks();
        let total = tasks.len();
        let complete = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Complete)
            .count();
        RoadmapStats {
            total,
            complete,
            pending: total - complete,
        }
    }

    /// Generate a compact state representation for AI context
    pub fn compact_state(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("# {}\n\n", self.title));

        for section in &self.sections {
            if section.tasks.is_empty() && section.subsections.is_empty() {
                continue;
            }
            out.push_str(&format_section_compact(section, 0));
        }

        out
    }

    /// Update raw content after modifications
    pub fn rebuild_raw(&mut self) {
        // For now, we do line-by-line updates
        // A more sophisticated version would rebuild from structure
    }
}

#[derive(Debug, Clone)]
pub struct RoadmapStats {
    pub total: usize,
    pub complete: usize,
    pub pending: usize,
}

// --- Parsing helpers ---

fn parse_heading(line: &str) -> Option<(u8, String)> {
    let trimmed = line.trim();
    let mut level = 0u8;
    for c in trimmed.chars() {
        if c == '#' {
            level += 1;
        } else {
            break;
        }
    }
    if level >= 2 && trimmed.len() > level as usize {
        let text = trimmed[level as usize..].trim().to_string();
        Some((level, text))
    } else {
        None
    }
}

fn parse_section(lines: &[&str], i: &mut usize, level: u8, heading: String) -> Section {
    let line_start = *i;
    let id = slugify(&heading);
    let mut theme = None;
    let mut tasks = Vec::new();
    let mut subsections = Vec::new();
    let mut raw_content = String::new();

    *i += 1;

    // Look for theme line (bold text after heading)
    while *i < lines.len() {
        let line = lines[*i].trim();
        if line.is_empty() {
            *i += 1;
            continue;
        }
        if line.starts_with("**") && line.ends_with("**") {
            theme = Some(line[2..line.len() - 2].to_string());
            *i += 1;
            break;
        }
        break;
    }

    // Parse content until next section of same or higher level
    while *i < lines.len() {
        let line = lines[*i];

        // Check for next section
        if let Some((next_level, next_heading)) = parse_heading(line) {
            if next_level <= level {
                // Same or higher level section - we're done
                break;
            } else {
                // Subsection
                let subsection = parse_section(lines, i, next_level, next_heading);
                subsections.push(subsection);
                continue;
            }
        }

        // Check for task
        if let Some(task) = parse_task_line(line, *i, &id) {
            tasks.push(task);
        } else {
            raw_content.push_str(line);
            raw_content.push('\n');
        }

        *i += 1;
    }

    // Build full paths for tasks
    for task in &mut tasks {
        task.path = format!("{}/{}", id, task.id);
    }

    Section {
        id,
        heading,
        level,
        theme,
        tasks,
        subsections,
        raw_content,
        line_start,
        line_end: *i,
    }
}

fn parse_task_line(line: &str, line_num: usize, parent_id: &str) -> Option<Task> {
    let trimmed = line.trim();

    // Count leading whitespace for indent level
    let indent = (line.len() - line.trim_start().len()) / 2;

    // Check for checkbox pattern
    let (status, rest) = if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
        (TaskStatus::Complete, trimmed[5..].trim())
    } else if trimmed.starts_with("- [ ]") {
        (TaskStatus::Pending, trimmed[5..].trim())
    } else {
        return None;
    };

    // Extract text (strip bold markers if present)
    let text = if rest.starts_with("**") {
        if let Some(end) = rest[2..].find("**") {
            rest[2..2 + end].to_string()
        } else {
            rest.to_string()
        }
    } else {
        // Take until newline or comment
        rest.split("<!--").next().unwrap_or(rest).trim().to_string()
    };

    // Check for explicit ID in comment
    let id = if let Some(comment_start) = rest.find("<!-- id:") {
        let after_marker = &rest[comment_start + 8..];
        if let Some(end) = after_marker.find("-->") {
            after_marker[..end].trim().to_string()
        } else {
            slugify(&text)
        }
    } else {
        slugify(&text)
    };

    Some(Task {
        id: id.clone(),
        path: String::new(), // Will be set by caller
        text,
        status,
        indent: indent as u8,
        line: line_num,
        children: Vec::new(),
    })
}

fn collect_tasks<'a>(section: &'a Section, tasks: &mut Vec<&'a Task>) {
    for task in &section.tasks {
        tasks.push(task);
        collect_tasks_recursive(task, tasks);
    }
    for subsection in &section.subsections {
        collect_tasks(subsection, tasks);
    }
}

fn collect_tasks_recursive<'a>(task: &'a Task, tasks: &mut Vec<&'a Task>) {
    for child in &task.children {
        tasks.push(child);
        collect_tasks_recursive(child, tasks);
    }
}

fn find_task_mut_in_section<'a>(section: &'a mut Section, path: &str) -> Option<&'a mut Task> {
    for task in &mut section.tasks {
        if task.path == path {
            return Some(task);
        }
    }
    for subsection in &mut section.subsections {
        if let Some(task) = find_task_mut_in_section(subsection, path) {
            return Some(task);
        }
    }
    None
}

fn format_section_compact(section: &Section, depth: usize) -> String {
    let mut out = String::new();
    let indent = "  ".repeat(depth);

    // Section header with stats
    let total = section.tasks.len();
    let complete = section
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Complete)
        .count();

    if total > 0 {
        out.push_str(&format!(
            "{}{} [{}/{}]\n",
            indent, section.heading, complete, total
        ));
        for task in &section.tasks {
            let marker = match task.status {
                TaskStatus::Complete => "✓",
                TaskStatus::Pending => "○",
            };
            out.push_str(&format!(
                "{}  {} {} ({})\n",
                indent, marker, task.text, task.path
            ));
        }
    } else if !section.subsections.is_empty() {
        out.push_str(&format!("{}{}\n", indent, section.heading));
    }

    for sub in &section.subsections {
        out.push_str(&format_section_compact(sub, depth + 1));
    }

    out
}

/// Convert text to a URL-safe slug
pub fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c == ' ' || c == '_' || c == '-' {
                '-'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Truncation detection"), "truncation-detection");
        assert_eq!(
            slugify("v0.5.0 — Bulletproof Apply"),
            "v0-5-0-bulletproof-apply"
        );
        assert_eq!(slugify("**Bold text**"), "bold-text");
    }

    #[test]
    fn test_parse_heading() {
        assert_eq!(parse_heading("## v0.5.0"), Some((2, "v0.5.0".to_string())));
        assert_eq!(
            parse_heading("### Subsection"),
            Some((3, "Subsection".to_string()))
        );
        assert_eq!(parse_heading("# Title"), None); // H1 not captured
        assert_eq!(parse_heading("Not a heading"), None);
    }

    #[test]
    fn test_parse_task_line() {
        let task = parse_task_line("- [ ] **Truncation detection**", 0, "test").unwrap();
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.text, "Truncation detection");
        assert_eq!(task.id, "truncation-detection");

        let task = parse_task_line("- [x] Done task", 0, "test").unwrap();
        assert_eq!(task.status, TaskStatus::Complete);

        let task = parse_task_line("- [ ] Custom ID <!-- id: my-custom-id -->", 0, "test").unwrap();
        assert_eq!(task.id, "my-custom-id");
    }
}
