// src/roadmap/cmd_parser.rs
use crate::roadmap::types::{Command, CommandBatch, MovePosition};

impl CommandBatch {
    #[must_use]
    pub fn parse(input: &str) -> Self {
        let mut commands = Vec::new();
        let mut errors = Vec::new();
        let content = extract_roadmap_block(input);

        for line in content.lines() {
            let line = line.trim();
            if is_skippable(line) {
                continue;
            }

            match parse_command_line(line) {
                Ok(cmd) => commands.push(cmd),
                Err(e) => {
                    if !line.is_empty() && !is_ignorable(line) {
                        errors.push(format!("Line '{}': {e}", truncate(line, 40)));
                    }
                }
            }
        }
        Self { commands, errors }
    }

    #[must_use]
    pub fn summary(&self) -> String {
        let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for cmd in &self.commands {
            *counts.entry(cmd_name(cmd)).or_insert(0) += 1;
        }

        if counts.is_empty() {
            return "No commands".to_string();
        }

        let parts: Vec<String> = counts.iter().map(|(k, v)| format!("{v} {k}")).collect();
        parts.join(", ")
    }
}

fn cmd_name(cmd: &Command) -> &'static str {
    match cmd {
        Command::Check { .. } => "CHECK",
        Command::Uncheck { .. } => "UNCHECK",
        Command::Add { .. } => "ADD",
        Command::Delete { .. } => "DELETE",
        Command::Update { .. } => "UPDATE",
        Command::Note { .. } => "NOTE",
        Command::Move { .. } => "MOVE",
        Command::ReplaceSection { .. } => "SECTION",
    }
}

fn extract_roadmap_block(input: &str) -> &str {
    if let Some(start) = input.find("===ROADMAP===") {
        let after = &input[start + 13..];
        return after.find("===END===").map_or(after, |end| &after[..end]);
    }
    input
}

fn is_skippable(line: &str) -> bool {
    line.is_empty() || line.starts_with('#') || line.starts_with("//")
}

// Refactored to group commands and lower complexity score
fn parse_command_line(line: &str) -> Result<Command, String> {
    let (cmd, args) = split_cmd(line).ok_or_else(|| "Empty command".to_string())?;

    if is_basic(cmd) {
        return parse_basic(cmd, args);
    }
    if is_content(cmd) {
        return parse_content(cmd, args);
    }
    if is_struct(cmd) {
        return parse_struct(cmd, args);
    }

    Err(format!("Unknown command: {cmd}"))
}

fn is_basic(cmd: &str) -> bool {
    matches!(cmd, "CHECK" | "UNCHECK" | "DELETE")
}
fn is_content(cmd: &str) -> bool {
    matches!(cmd, "ADD" | "UPDATE" | "NOTE")
}
fn is_struct(cmd: &str) -> bool {
    matches!(cmd, "MOVE" | "SECTION")
}

fn parse_basic(cmd: &str, args: &str) -> Result<Command, String> {
    let path = req_path(args)?;
    match cmd {
        "CHECK" => Ok(Command::Check { path }),
        "UNCHECK" => Ok(Command::Uncheck { path }),
        "DELETE" => Ok(Command::Delete { path }),
        _ => unreachable!(),
    }
}

fn parse_content(cmd: &str, args: &str) -> Result<Command, String> {
    match cmd {
        "ADD" => parse_add(args),
        "UPDATE" => parse_update(args),
        "NOTE" => parse_note(args),
        _ => unreachable!(),
    }
}

fn parse_struct(cmd: &str, args: &str) -> Result<Command, String> {
    match cmd {
        "MOVE" => parse_move(args),
        "SECTION" => parse_section(args),
        _ => unreachable!(),
    }
}

fn split_cmd(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.splitn(2, ' ');
    let cmd = parts.next()?;
    let args = parts.next().unwrap_or("");
    if cmd.is_empty() {
        return None;
    }
    Some((cmd, args))
}

fn req_path(args: &str) -> Result<String, String> {
    let path = args.trim();
    if path.is_empty() {
        return Err("Requires task path".into());
    }
    Ok(path.to_string())
}

fn parse_add(args: &str) -> Result<Command, String> {
    let (parent, rest) = split_first_word(args);
    if parent.is_empty() {
        return Err("ADD needs parent".into());
    }
    let (text, after) = parse_quoted_with_after(rest)?;
    Ok(Command::Add {
        parent: parent.into(),
        text,
        after,
    })
}

fn parse_update(args: &str) -> Result<Command, String> {
    let (path, rest) = split_first_word(args);
    if path.is_empty() {
        return Err("UPDATE needs path".into());
    }
    Ok(Command::Update {
        path: path.into(),
        text: parse_quoted(rest)?,
    })
}

fn parse_note(args: &str) -> Result<Command, String> {
    let (path, rest) = split_first_word(args);
    if path.is_empty() {
        return Err("NOTE needs path".into());
    }
    Ok(Command::Note {
        path: path.into(),
        note: parse_quoted(rest)?,
    })
}

fn parse_move(args: &str) -> Result<Command, String> {
    let parts: Vec<&str> = args.split_whitespace().collect();
    if parts.len() < 3 {
        return Err("MOVE: path AFTER|BEFORE target".into());
    }

    let pos = match parts[1].to_uppercase().as_str() {
        "AFTER" => MovePosition::After(parts[2].into()),
        "BEFORE" => MovePosition::Before(parts[2].into()),
        _ => return Err("Invalid position".into()),
    };
    Ok(Command::Move {
        path: parts[0].into(),
        position: pos,
    })
}

fn parse_section(args: &str) -> Result<Command, String> {
    let id = args.trim();
    if id.is_empty() {
        return Err("SECTION needs ID".into());
    }
    Ok(Command::ReplaceSection {
        id: id.into(),
        content: String::new(),
    })
}

fn split_first_word(s: &str) -> (&str, &str) {
    s.trim()
        .split_once(char::is_whitespace)
        .map_or((s.trim(), ""), |(h, t)| (h, t.trim()))
}

fn parse_quoted(s: &str) -> Result<String, String> {
    let s = s.trim();
    if let Some(stripped) = s.strip_prefix('"') {
        stripped
            .find('"')
            .map(|end| stripped[..end].to_string())
            .ok_or_else(|| "Unclosed quote".into())
    } else {
        Ok(s.to_string())
    }
}

fn parse_quoted_with_after(s: &str) -> Result<(String, Option<String>), String> {
    let (text, rest) = extract_quoted_text(s)?;

    let after = if let Some(stripped) = rest.strip_prefix("AFTER ") {
        Some(stripped.trim().to_string())
    } else {
        rest.strip_prefix("after ")
            .map(|stripped| stripped.trim().to_string())
    };

    Ok((text, after))
}

fn extract_quoted_text(s: &str) -> Result<(String, &str), String> {
    let s = s.trim();
    if let Some(stripped) = s.strip_prefix('"') {
        let end = stripped.find('"').ok_or("Unclosed quote")?;
        Ok((stripped[..end].to_string(), stripped[end + 1..].trim()))
    } else if let Some((text, rest)) = s.split_once(" AFTER ") {
        Ok((text.trim().to_string(), rest.trim()))
    } else {
        Ok((s.to_string(), ""))
    }
}

fn is_ignorable(line: &str) -> bool {
    let u = line.to_uppercase();
    u.starts_with("===")
        || u.starts_with("---")
        || u.starts_with("```")
        || u == "ROADMAP"
        || u == "END"
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}
