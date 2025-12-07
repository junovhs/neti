// src/roadmap_v2/parser.rs
use super::types::{RoadmapCommand, Task, TaskStatus, TaskUpdate};
use anyhow::{anyhow, bail, Result};

/// Parses roadmap commands from the ===ROADMAP=== block(s) in AI output.
/// Handles multiple blocks and command aggregation.
///
/// # Errors
/// Returns error if command syntax is invalid.
pub fn parse_commands(input: &str) -> Result<Vec<RoadmapCommand>> {
    let blocks = extract_roadmap_blocks(input);
    if blocks.is_empty() {
        return Ok(vec![]);
    }

    let mut commands = Vec::new();

    for block in blocks {
        let mut block_cmds = parse_block_content(&block)?;
        commands.append(&mut block_cmds);
    }

    Ok(commands)
}

/// Extracts all content blocks delimited by ===ROADMAP===.
fn extract_roadmap_blocks(input: &str) -> Vec<String> {
    let marker = "===ROADMAP===";
    let mut blocks = Vec::new();
    let mut current_pos = 0;

    while let Some(start_offset) = input[current_pos..].find(marker) {
        let content_start = current_pos + start_offset + marker.len();

        if let Some(end_offset) = input[content_start..].find(marker) {
            let content_end = content_start + end_offset;
            let block = input[content_start..content_end].trim();
            if !block.is_empty() {
                blocks.push(block.to_string());
            }
            current_pos = content_end + marker.len();
        } else {
            break;
        }
    }
    blocks
}

fn parse_block_content(block: &str) -> Result<Vec<RoadmapCommand>> {
    let mut commands = Vec::new();
    let mut current_block = String::new();

    for line in block.lines() {
        let trimmed = clean_line(line);

        if trimmed.is_empty() {
            continue;
        }

        if is_command_keyword(trimmed) {
            if !current_block.is_empty() {
                commands.push(parse_single_command(&current_block)?);
            }
            current_block = trimmed.to_string();
        } else {
            if !current_block.is_empty() {
                current_block.push('\n');
            }
            current_block.push_str(line);
        }
    }

    if !current_block.is_empty() {
        commands.push(parse_single_command(&current_block)?);
    }

    Ok(commands)
}

fn clean_line(line: &str) -> &str {
    line.split('#').next().unwrap_or("").trim()
}

fn is_command_keyword(line: &str) -> bool {
    let upper = clean_line(line).to_uppercase();
    matches!(
        upper.as_str(),
        "CHECK" | "UNCHECK" | "ADD" | "UPDATE" | "DELETE"
    )
}

fn parse_single_command(block: &str) -> Result<RoadmapCommand> {
    let lines: Vec<&str> = block.lines().collect();
    let keyword_line = lines.first().copied().unwrap_or_default();
    let keyword = clean_line(keyword_line).to_uppercase();

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
    for line in lines {
        let trimmed = clean_line(line);
        if let Some((k, v)) = trimmed.split_once('=') {
            if k.trim() == key {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    // Helper to obscure markers from the main tool's parser
    fn make_block(content: &str) -> String {
        format!("\n{}\n{}\n{}\n", "===ROADMAP===", content, "===ROADMAP===")
    }

    #[test]
    fn test_parse_check() -> Result<()> {
        let input = make_block("CHECK\nid = my-task");
        let cmds = parse_commands(&input)?;
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], RoadmapCommand::Check { ref id } if id == "my-task"));
        Ok(())
    }

    #[test]
    fn test_parse_add() -> Result<()> {
        let input = make_block("ADD\nid = new-task\ntext = Implement feature X\nsection = v0.2.0");
        let cmds = parse_commands(&input)?;
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], RoadmapCommand::Add(_)));
        Ok(())
    }

    #[test]
    fn test_parse_multiple_same_block() -> Result<()> {
        let input = make_block("CHECK\nid = task-1\nCHECK\nid = task-2");
        let cmds = parse_commands(&input)?;
        assert_eq!(cmds.len(), 2);
        Ok(())
    }

    #[test]
    fn test_parse_multiple_blocks() -> Result<()> {
        let b1 = make_block("CHECK\nid = task-1");
        let b2 = make_block("CHECK\nid = task-2");
        let input = format!("{b1}\nSome text...\n{b2}");

        let cmds = parse_commands(&input)?;
        assert_eq!(cmds.len(), 2);
        if let RoadmapCommand::Check { id } = &cmds[0] {
            assert_eq!(id, "task-1");
        }
        if let RoadmapCommand::Check { id } = &cmds[1] {
            assert_eq!(id, "task-2");
        }
        Ok(())
    }

    #[test]
    fn test_with_comments_and_spacing() -> Result<()> {
        let input = make_block(
            "CHECK # First one\nid=task-1 # id comment\nCHECK\nid = task-2",
        );
        let cmds = parse_commands(&input)?;
        assert_eq!(cmds.len(), 2);
        Ok(())
    }
}