// src/roadmap_v2/parser.rs
use super::types::{RoadmapCommand, Task, TaskStatus, TaskUpdate};
use anyhow::{anyhow, bail, Result};

/// Parses roadmap commands from the ===ROADMAP=== block in AI output.
///
/// # Errors
/// Returns error if command syntax is invalid.
pub fn parse_commands(input: &str) -> Result<Vec<RoadmapCommand>> {
    let Some(block) = extract_roadmap_block(input) else {
        return Ok(vec![]);
    };

    let mut commands = Vec::new();
    let mut current_block = String::new();

    for line in block.lines() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Check if this is a command keyword (starts a new block)
        if is_command_keyword(trimmed) {
            // Process the previous block if any
            if !current_block.is_empty() {
                commands.push(parse_single_block(&current_block)?);
            }
            current_block = trimmed.to_string();
        } else {
            // Add to current block
            if !current_block.is_empty() {
                current_block.push('\n');
            }
            current_block.push_str(trimmed);
        }
    }

    // Process final block
    if !current_block.is_empty() {
        commands.push(parse_single_block(&current_block)?);
    }

    Ok(commands)
}

fn extract_roadmap_block(input: &str) -> Option<&str> {
    let start_marker = "===ROADMAP===";
    let start = input.find(start_marker)? + start_marker.len();
    let rest = &input[start..];
    let end = rest.find(start_marker)?;
    Some(rest[..end].trim())
}

fn is_command_keyword(line: &str) -> bool {
    let upper = line.to_uppercase();
    matches!(
        upper.as_str(),
        "CHECK" | "UNCHECK" | "ADD" | "UPDATE" | "DELETE"
    )
}

fn parse_single_block(block: &str) -> Result<RoadmapCommand> {
    let lines: Vec<&str> = block.lines().collect();
    let keyword = lines
        .first()
        .map(|s| s.trim().to_uppercase())
        .unwrap_or_default();

    match keyword.as_str() {
        "CHECK" => parse_check(&lines),
        "UNCHECK" => parse_uncheck(&lines),
        "ADD" => parse_add(&lines),
        "UPDATE" => parse_update(&lines),
        "DELETE" => parse_delete(&lines),
        "" => bail!("Empty command block"),
        other => bail!("Unknown roadmap command: {other}"),
    }
}

fn parse_check(lines: &[&str]) -> Result<RoadmapCommand> {
    let id = require_field(lines, "id")?;
    Ok(RoadmapCommand::Check { id })
}

fn parse_uncheck(lines: &[&str]) -> Result<RoadmapCommand> {
    let id = require_field(lines, "id")?;
    Ok(RoadmapCommand::Uncheck { id })
}

fn parse_delete(lines: &[&str]) -> Result<RoadmapCommand> {
    let id = require_field(lines, "id")?;
    Ok(RoadmapCommand::Delete { id })
}

fn parse_add(lines: &[&str]) -> Result<RoadmapCommand> {
    let id = require_field(lines, "id")?;
    let text = require_field(lines, "text")?;
    let section = require_field(lines, "section")?;
    let test_anchor = get_field(lines, "test");
    let group = get_field(lines, "group");

    Ok(RoadmapCommand::Add(Task {
        id,
        text,
        status: TaskStatus::Pending,
        section,
        test: test_anchor,
        group,
        order: 0,
    }))
}

fn parse_update(lines: &[&str]) -> Result<RoadmapCommand> {
    let id = require_field(lines, "id")?;
    let fields = TaskUpdate {
        text: get_field(lines, "text"),
        test: get_field(lines, "test"),
        section: get_field(lines, "section"),
        group: get_field(lines, "group"),
    };
    Ok(RoadmapCommand::Update { id, fields })
}

fn require_field(lines: &[&str], key: &str) -> Result<String> {
    get_field(lines, key).ok_or_else(|| anyhow!("Missing required field: {key}"))
}

fn get_field(lines: &[&str], key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with(&prefix) {
            return Some(trimmed[prefix.len()..].trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_parse_check() -> Result<()> {
        let input = r"
===ROADMAP===
CHECK
id = my-task
===ROADMAP===
";
        let cmds = parse_commands(input)?;
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], RoadmapCommand::Check { ref id } if id == "my-task"));
        Ok(())
    }

    #[test]
    fn test_parse_add() -> Result<()> {
        let input = r"
===ROADMAP===
ADD
id = new-task
text = Implement feature X
section = v0.2.0
===ROADMAP===
";
        let cmds = parse_commands(input)?;
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], RoadmapCommand::Add(_)));
        Ok(())
    }

    #[test]
    fn test_parse_multiple() -> Result<()> {
        let input = r"
===ROADMAP===
CHECK
id = task-1
CHECK
id = task-2
ADD
id = task-3
text = New thing
section = v1.0.0
===ROADMAP===
";
        let cmds = parse_commands(input)?;
        assert_eq!(cmds.len(), 3);
        Ok(())
    }

    #[test]
    fn test_no_roadmap_block() -> Result<()> {
        let input = "Just some code without roadmap";
        let cmds = parse_commands(input)?;
        assert!(cmds.is_empty());
        Ok(())
    }
}