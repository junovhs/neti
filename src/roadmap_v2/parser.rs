// src/roadmap_v2/parser.rs
use super::types::{AddCommand, AfterTarget, RoadmapCommand, Task, TaskStatus, TaskUpdate};
use anyhow::{anyhow, bail, Result};

/// Parses roadmap commands from the ===ROADMAP=== block(s) in AI output.
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

fn extract_roadmap_blocks(input: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut state = BlockState::default();

    for line in input.lines() {
        process_line_for_blocks(line, &mut state, &mut blocks);
    }

    blocks
}

#[derive(Default)]
struct BlockState {
    capturing: bool,
    current_block: String,
}

fn process_line_for_blocks(line: &str, state: &mut BlockState, blocks: &mut Vec<String>) {
    let marker = "===ROADMAP===";
    let trimmed = line.trim();

    if trimmed == marker {
        if state.capturing {
            if !state.current_block.trim().is_empty() {
                blocks.push(state.current_block.clone());
            }
            state.current_block.clear();
            state.capturing = false;
        } else {
            state.capturing = true;
        }
    } else if state.capturing {
        state.current_block.push_str(line);
        state.current_block.push('\n');
    }
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
    let after = parse_after_field(lines);

    let task = Task {
        id,
        text,
        status: TaskStatus::Pending,
        section,
        test: get_field(lines, "test"),
        group: get_field(lines, "group"),
        order: 0,
    };

    Ok(RoadmapCommand::Add(AddCommand { task, after }))
}

fn parse_after_field(lines: &[&str]) -> AfterTarget {
    let Some(value) = get_field(lines, "after") else {
        return AfterTarget::End;
    };

    let upper = value.to_uppercase();

    if upper == "PREVIOUS" {
        return AfterTarget::Previous;
    }

    if upper.starts_with("TEXT ") {
        let text = extract_quoted(&value[5..]);
        return AfterTarget::Text(text);
    }

    if upper.starts_with("LINE ") {
        if let Ok(n) = value[5..].trim().parse::<usize>() {
            return AfterTarget::Line(n);
        }
    }

    // Default: treat as task ID
    AfterTarget::Id(value)
}

fn extract_quoted(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() > 1 {
        return trimmed[1..trimmed.len() - 1].to_string();
    }
    trimmed.to_string()
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
    let value = get_field(lines, key).ok_or_else(|| anyhow!("Missing required field: {key}"))?;
    if value.trim().is_empty() {
        bail!("Field '{key}' cannot be empty");
    }
    Ok(value)
}

fn get_field(lines: &[&str], key: &str) -> Option<String> {
    for line in lines {
        let trimmed = clean_line(line);
        let Some((k, v)) = trimmed.split_once('=') else {
            continue;
        };
        if k.trim() != key {
            continue;
        }
        let value = v.trim();
        if value.is_empty() {
            return None;
        }
        return Some(value.to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_block(content: &str) -> String {
        format!("\n{}\n{}\n{}\n", "===ROADMAP===", content, "===ROADMAP===")
    }

    #[test]
    fn test_parse_check() -> Result<()> {
        let input = make_block("CHECK\nid = my-task");
        let cmds = parse_commands(&input)?;
        assert!(matches!(cmds[0], RoadmapCommand::Check { ref id } if id == "my-task"));
        Ok(())
    }

    #[test]
    fn test_add_after_previous() -> Result<()> {
        let input = make_block("ADD\nid = t\ntext = X\nsection = v1\nafter = PREVIOUS");
        let cmds = parse_commands(&input)?;
        match &cmds[0] {
            RoadmapCommand::Add(cmd) => {
                assert!(matches!(cmd.after, AfterTarget::Previous));
            }
            _ => panic!("Expected Add command"),
        }
        Ok(())
    }

    #[test]
    fn test_add_after_text() -> Result<()> {
        let input = make_block("ADD\nid = t\ntext = X\nsection = v1\nafter = TEXT \"foo bar\"");
        let cmds = parse_commands(&input)?;
        match &cmds[0] {
            RoadmapCommand::Add(cmd) => {
                assert!(matches!(&cmd.after, AfterTarget::Text(t) if t == "foo bar"));
            }
            _ => panic!("Expected Add command"),
        }
        Ok(())
    }

    #[test]
    fn test_add_after_line() -> Result<()> {
        let input = make_block("ADD\nid = t\ntext = X\nsection = v1\nafter = LINE 5");
        let cmds = parse_commands(&input)?;
        match &cmds[0] {
            RoadmapCommand::Add(cmd) => {
                assert!(matches!(cmd.after, AfterTarget::Line(5)));
            }
            _ => panic!("Expected Add command"),
        }
        Ok(())
    }
}