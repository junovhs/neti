//! Command parsing and execution
//!
//! Handles the AI command format for surgical roadmap updates.

use crate::parser::{slugify, Roadmap, TaskStatus};
use std::fmt;

/// A single command from AI
#[derive(Debug, Clone)]
pub enum Command {
    /// Mark a task complete: CHECK v0.5.0/truncation-detection
    Check { path: String },

    /// Mark a task incomplete: UNCHECK v0.5.0/truncation-detection
    Uncheck { path: String },

    /// Add a new task: ADD v0.5.0 "New task description"
    Add {
        parent: String,
        text: String,
        after: Option<String>,
    },

    /// Delete a task: DELETE v0.5.0/old-task
    Delete { path: String },

    /// Update task text: UPDATE v0.5.0/task-id "New description"
    Update { path: String, text: String },

    /// Add a note/comment to a task: NOTE v0.5.0/task-id "Implementation note"
    Note { path: String, note: String },

    /// Move a task: MOVE v0.5.0/task-id AFTER v0.5.0/other-task
    Move {
        path: String,
        position: MovePosition,
    },

    /// Replace entire section content: SECTION v0.5.0 ... END
    ReplaceSection { id: String, content: String },
}

#[derive(Debug, Clone)]
pub enum MovePosition {
    After(String),
    Before(String),
}

/// A batch of commands parsed from AI output
#[derive(Debug, Clone)]
pub struct CommandBatch {
    pub commands: Vec<Command>,
    pub errors: Vec<String>,
}

/// Result of applying a command
#[derive(Debug)]
pub enum ApplyResult {
    Success(String),
    NotFound(String),
    Error(String),
}

impl fmt::Display for ApplyResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApplyResult::Success(msg) => write!(f, "✓ {}", msg),
            ApplyResult::NotFound(msg) => write!(f, "✗ Not found: {}", msg),
            ApplyResult::Error(msg) => write!(f, "✗ Error: {}", msg),
        }
    }
}

impl CommandBatch {
    /// Parse commands from AI output text
    ///
    /// Expected format:
    /// ```text
    /// ===ROADMAP===
    /// CHECK v0.5.0/path-safety-validation
    /// ADD v0.5.0 "New task" AFTER truncation-detection
    /// UPDATE v0.5.0/some-task "Updated description"
    /// ===END===
    /// ```
    pub fn parse(input: &str) -> Self {
        let mut commands = Vec::new();
        let mut errors = Vec::new();

        // Find command block
        let content = if let Some(start) = input.find("===ROADMAP===") {
            let after_start = &input[start + 13..];
            if let Some(end) = after_start.find("===END===") {
                &after_start[..end]
            } else {
                // No end marker, take everything after start
                after_start
            }
        } else {
            // No markers, try to parse the whole thing
            input
        };

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                continue;
            }

            match parse_command_line(line) {
                Ok(cmd) => commands.push(cmd),
                Err(e) => {
                    if !line.is_empty() && !is_ignorable(line) {
                        errors.push(format!("Line '{}': {}", truncate(line, 40), e));
                    }
                }
            }
        }

        CommandBatch { commands, errors }
    }

    /// Check if the batch is valid (has commands, no critical errors)
    pub fn is_valid(&self) -> bool {
        !self.commands.is_empty()
    }

    /// Get a summary of what will be done
    pub fn summary(&self) -> String {
        let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();

        for cmd in &self.commands {
            let name = match cmd {
                Command::Check { .. } => "CHECK",
                Command::Uncheck { .. } => "UNCHECK",
                Command::Add { .. } => "ADD",
                Command::Delete { .. } => "DELETE",
                Command::Update { .. } => "UPDATE",
                Command::Note { .. } => "NOTE",
                Command::Move { .. } => "MOVE",
                Command::ReplaceSection { .. } => "SECTION",
            };
            *counts.entry(name).or_insert(0) += 1;
        }

        let parts: Vec<String> = counts.iter().map(|(k, v)| format!("{} {}", v, k)).collect();

        if parts.is_empty() {
            "No commands".to_string()
        } else {
            parts.join(", ")
        }
    }
}

fn parse_command_line(line: &str) -> Result<Command, String> {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }

    let cmd = parts[0].to_uppercase();
    let args = parts.get(1).map(|s| *s).unwrap_or("");

    match cmd.as_str() {
        "CHECK" => {
            let path = args.trim().to_string();
            if path.is_empty() {
                return Err("CHECK requires a task path".to_string());
            }
            Ok(Command::Check { path })
        }

        "UNCHECK" => {
            let path = args.trim().to_string();
            if path.is_empty() {
                return Err("UNCHECK requires a task path".to_string());
            }
            Ok(Command::Uncheck { path })
        }

        "ADD" => {
            // Format: ADD parent "task text" [AFTER sibling]
            let (parent, rest) = split_first_word(args);
            if parent.is_empty() {
                return Err("ADD requires a parent section".to_string());
            }

            let (text, after) = parse_quoted_with_after(rest)?;
            Ok(Command::Add {
                parent: parent.to_string(),
                text,
                after,
            })
        }

        "DELETE" => {
            let path = args.trim().to_string();
            if path.is_empty() {
                return Err("DELETE requires a task path".to_string());
            }
            Ok(Command::Delete { path })
        }

        "UPDATE" => {
            // Format: UPDATE path "new text"
            let (path, rest) = split_first_word(args);
            if path.is_empty() {
                return Err("UPDATE requires a task path".to_string());
            }
            let text = parse_quoted(rest)?;
            Ok(Command::Update {
                path: path.to_string(),
                text,
            })
        }

        "NOTE" => {
            // Format: NOTE path "note text"
            let (path, rest) = split_first_word(args);
            if path.is_empty() {
                return Err("NOTE requires a task path".to_string());
            }
            let note = parse_quoted(rest)?;
            Ok(Command::Note {
                path: path.to_string(),
                note,
            })
        }

        "MOVE" => {
            // Format: MOVE path AFTER|BEFORE target
            let parts: Vec<&str> = args.split_whitespace().collect();
            if parts.len() < 3 {
                return Err("MOVE requires: path AFTER|BEFORE target".to_string());
            }
            let path = parts[0].to_string();
            let position = match parts[1].to_uppercase().as_str() {
                "AFTER" => MovePosition::After(parts[2].to_string()),
                "BEFORE" => MovePosition::Before(parts[2].to_string()),
                _ => return Err("MOVE position must be AFTER or BEFORE".to_string()),
            };
            Ok(Command::Move { path, position })
        }

        "SECTION" => {
            // Multi-line: everything until END on its own line
            // For single-line parsing, we'd need the full content
            // This is a simplified version
            let id = args.trim().to_string();
            if id.is_empty() {
                return Err("SECTION requires a section ID".to_string());
            }
            // Content would be accumulated by the batch parser
            Ok(Command::ReplaceSection {
                id,
                content: String::new(),
            })
        }

        _ => Err(format!("Unknown command: {}", cmd)),
    }
}

fn split_first_word(s: &str) -> (&str, &str) {
    let s = s.trim();
    if let Some(idx) = s.find(|c: char| c.is_whitespace()) {
        (&s[..idx], s[idx..].trim())
    } else {
        (s, "")
    }
}

fn parse_quoted(s: &str) -> Result<String, String> {
    let s = s.trim();
    if s.starts_with('"') {
        if let Some(end) = s[1..].find('"') {
            Ok(s[1..=end].to_string())
        } else {
            Err("Unclosed quote".to_string())
        }
    } else {
        // Unquoted - take the whole thing
        Ok(s.to_string())
    }
}

fn parse_quoted_with_after(s: &str) -> Result<(String, Option<String>), String> {
    let s = s.trim();
    if s.starts_with('"') {
        if let Some(end) = s[1..].find('"') {
            let text = s[1..=end].to_string();
            let rest = s[end + 2..].trim();

            let after = if rest.to_uppercase().starts_with("AFTER ") {
                Some(rest[6..].trim().to_string())
            } else {
                None
            };

            Ok((text, after))
        } else {
            Err("Unclosed quote".to_string())
        }
    } else {
        // Try to find AFTER keyword
        if let Some(idx) = s.to_uppercase().find(" AFTER ") {
            let text = s[..idx].trim().to_string();
            let after = s[idx + 7..].trim().to_string();
            Ok((text, Some(after)))
        } else {
            Ok((s.to_string(), None))
        }
    }
}

fn is_ignorable(line: &str) -> bool {
    let line = line.to_uppercase();
    line.starts_with("===")
        || line.starts_with("---")
        || line.starts_with("```")
        || line == "ROADMAP"
        || line == "END"
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

/// Apply a batch of commands to a roadmap
pub fn apply_commands(roadmap: &mut Roadmap, batch: &CommandBatch) -> Vec<ApplyResult> {
    let mut results = Vec::new();

    for cmd in &batch.commands {
        let result = apply_single_command(roadmap, cmd);
        results.push(result);
    }

    results
}

fn apply_single_command(roadmap: &mut Roadmap, cmd: &Command) -> ApplyResult {
    match cmd {
        Command::Check { path } => apply_status_change(roadmap, path, TaskStatus::Complete),
        Command::Uncheck { path } => apply_status_change(roadmap, path, TaskStatus::Pending),
        Command::Add {
            parent,
            text,
            after,
        } => apply_add(roadmap, parent, text, after.as_deref()),
        Command::Delete { path } => apply_delete(roadmap, path),
        Command::Update { path, text } => apply_update(roadmap, path, text),
        Command::Note { path, note } => apply_note(roadmap, path, note),
        Command::Move { path, position } => apply_move(roadmap, path, position),
        Command::ReplaceSection { id, content } => apply_section_replace(roadmap, id, content),
    }
}

fn apply_status_change(roadmap: &mut Roadmap, path: &str, status: TaskStatus) -> ApplyResult {
    // Find the task in the raw content and update the checkbox
    let lines: Vec<&str> = roadmap.raw.lines().collect();

    // Find task by path
    if let Some(task) = roadmap.find_task(path) {
        let line_idx = task.line;
        if line_idx < lines.len() {
            let line = lines[line_idx];
            let new_line = match status {
                TaskStatus::Complete => line.replace("- [ ]", "- [x]"),
                TaskStatus::Pending => line.replace("- [x]", "- [ ]").replace("- [X]", "- [ ]"),
            };

            // Rebuild raw content
            let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
            new_lines[line_idx] = new_line;
            roadmap.raw = new_lines.join("\n");

            let action = if status == TaskStatus::Complete {
                "Checked"
            } else {
                "Unchecked"
            };
            return ApplyResult::Success(format!("{}: {}", action, path));
        }
    }

    // Fallback: search by slugified text
    let search_id = path.split('/').last().unwrap_or(path);
    for (idx, line) in lines.iter().enumerate() {
        if line.contains("- [ ]") || line.contains("- [x]") || line.contains("- [X]") {
            let line_slug = slugify(line);
            if line_slug.contains(search_id) {
                let new_line = match status {
                    TaskStatus::Complete => line.replace("- [ ]", "- [x]"),
                    TaskStatus::Pending => line.replace("- [x]", "- [ ]").replace("- [X]", "- [ ]"),
                };

                let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
                new_lines[idx] = new_line;
                roadmap.raw = new_lines.join("\n");

                let action = if status == TaskStatus::Complete {
                    "Checked"
                } else {
                    "Unchecked"
                };
                return ApplyResult::Success(format!("{}: {}", action, path));
            }
        }
    }

    ApplyResult::NotFound(path.to_string())
}

fn apply_add(roadmap: &mut Roadmap, parent: &str, text: &str, after: Option<&str>) -> ApplyResult {
    let lines: Vec<&str> = roadmap.raw.lines().collect();
    let parent_slug = slugify(parent);

    // Find the parent section
    let mut in_section = false;
    let mut insert_idx = None;
    let mut last_task_idx = None;

    for (idx, line) in lines.iter().enumerate() {
        // Check if we're entering the target section
        if line.starts_with("## ") || line.starts_with("### ") {
            let heading_slug = slugify(line);
            if heading_slug.contains(&parent_slug) {
                in_section = true;
                insert_idx = Some(idx + 1);
                continue;
            } else if in_section {
                // We've left the section
                break;
            }
        }

        if in_section {
            // Track task lines
            if line.trim().starts_with("- [") {
                last_task_idx = Some(idx);

                // If we're looking for a specific "after" task
                if let Some(after_id) = after {
                    let line_slug = slugify(line);
                    if line_slug.contains(&slugify(after_id)) {
                        insert_idx = Some(idx + 1);
                    }
                }
            }
        }
    }

    // Default to after last task in section, or just after section header
    let insert_at = last_task_idx.map(|i| i + 1).or(insert_idx);

    if let Some(idx) = insert_at {
        let new_task = format!("- [ ] **{}**", text);
        let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
        new_lines.insert(idx, new_task);
        roadmap.raw = new_lines.join("\n");

        ApplyResult::Success(format!("Added: {} in {}", text, parent))
    } else {
        ApplyResult::NotFound(format!("Section: {}", parent))
    }
}

fn apply_delete(roadmap: &mut Roadmap, path: &str) -> ApplyResult {
    let lines: Vec<&str> = roadmap.raw.lines().collect();
    let search_id = path.split('/').last().unwrap_or(path);
    let search_slug = slugify(search_id);

    for (idx, line) in lines.iter().enumerate() {
        if line.trim().starts_with("- [") {
            let line_slug = slugify(line);
            if line_slug.contains(&search_slug) {
                let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
                new_lines.remove(idx);
                roadmap.raw = new_lines.join("\n");
                return ApplyResult::Success(format!("Deleted: {}", path));
            }
        }
    }

    ApplyResult::NotFound(path.to_string())
}

fn apply_update(roadmap: &mut Roadmap, path: &str, new_text: &str) -> ApplyResult {
    let lines: Vec<&str> = roadmap.raw.lines().collect();
    let search_id = path.split('/').last().unwrap_or(path);
    let search_slug = slugify(search_id);

    for (idx, line) in lines.iter().enumerate() {
        if line.trim().starts_with("- [") {
            let line_slug = slugify(line);
            if line_slug.contains(&search_slug) {
                // Preserve checkbox state and indentation
                let indent = line.len() - line.trim_start().len();
                let indent_str = &line[..indent];
                let checkbox = if line.contains("- [x]") || line.contains("- [X]") {
                    "- [x]"
                } else {
                    "- [ ]"
                };

                let new_line = format!("{}{} **{}**", indent_str, checkbox, new_text);
                let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
                new_lines[idx] = new_line;
                roadmap.raw = new_lines.join("\n");

                return ApplyResult::Success(format!("Updated: {}", path));
            }
        }
    }

    ApplyResult::NotFound(path.to_string())
}

fn apply_note(roadmap: &mut Roadmap, path: &str, note: &str) -> ApplyResult {
    let lines: Vec<&str> = roadmap.raw.lines().collect();
    let search_id = path.split('/').last().unwrap_or(path);
    let search_slug = slugify(search_id);

    for (idx, line) in lines.iter().enumerate() {
        if line.trim().starts_with("- [") {
            let line_slug = slugify(line);
            if line_slug.contains(&search_slug) {
                // Find where to insert the note (after task, before next task/section)
                let indent = line.len() - line.trim_start().len();
                let note_indent = " ".repeat(indent + 2);
                let note_line = format!("{}*{}*", note_indent, note);

                let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
                new_lines.insert(idx + 1, note_line);
                roadmap.raw = new_lines.join("\n");

                return ApplyResult::Success(format!("Added note to: {}", path));
            }
        }
    }

    ApplyResult::NotFound(path.to_string())
}

fn apply_move(_roadmap: &mut Roadmap, path: &str, _position: &MovePosition) -> ApplyResult {
    // TODO: Implement move
    ApplyResult::Error(format!("MOVE not yet implemented for: {}", path))
}

fn apply_section_replace(_roadmap: &mut Roadmap, id: &str, _content: &str) -> ApplyResult {
    // TODO: Implement section replace
    ApplyResult::Error(format!("SECTION replace not yet implemented for: {}", id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_check() {
        let batch = CommandBatch::parse("CHECK v0.5.0/truncation-detection");
        assert_eq!(batch.commands.len(), 1);
        assert!(
            matches!(&batch.commands[0], Command::Check { path } if path == "v0.5.0/truncation-detection")
        );
    }

    #[test]
    fn test_parse_add() {
        let batch = CommandBatch::parse(r#"ADD v0.5.0 "New task description""#);
        assert_eq!(batch.commands.len(), 1);
        assert!(
            matches!(&batch.commands[0], Command::Add { parent, text, after: None }
            if parent == "v0.5.0" && text == "New task description")
        );
    }

    #[test]
    fn test_parse_add_with_after() {
        let batch = CommandBatch::parse(r#"ADD v0.5.0 "New task" AFTER truncation"#);
        assert_eq!(batch.commands.len(), 1);
        assert!(
            matches!(&batch.commands[0], Command::Add { after: Some(a), .. } if a == "truncation")
        );
    }

    #[test]
    fn test_parse_batch_with_markers() {
        let input = r#"
Here's what I'll do:

===ROADMAP===
CHECK v0.5.0/path-safety
ADD v0.5.0 "New feature"
===END===

Let me know if you need anything else!
"#;
        let batch = CommandBatch::parse(input);
        assert_eq!(batch.commands.len(), 2);
    }

    #[test]
    fn test_summary() {
        let batch = CommandBatch::parse(
            r#"
CHECK a
CHECK b
ADD section "task"
DELETE old
"#,
        );
        let summary = batch.summary();
        assert!(summary.contains("2 CHECK"));
        assert!(summary.contains("1 ADD"));
        assert!(summary.contains("1 DELETE"));
    }
}
