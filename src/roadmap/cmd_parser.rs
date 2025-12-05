// src/roadmap/cmd_parser.rs
use crate::roadmap::str_utils;
use crate::roadmap::types::{Command, CommandBatch, MovePosition};

impl CommandBatch {
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
                    if !line.is_empty() && !str_utils::is_ignorable(line) {
                        errors.push(format!("Line '{}': {e}", str_utils::truncate(line, 40)));
                    }
                }
            }
        }
        Self { commands, errors }
    }

    pub fn summary(&self) -> String {
        let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for cmd in &self.commands {
            *counts.entry(cmd_name(cmd)).or_insert(0) += 1;
        }
        if counts.is_empty() { return "No commands".to_string(); }
        counts.iter().map(|(k, v)| format!("{v} {k}")).collect::<Vec<_>>().join(", ")
    }
}

fn cmd_name(cmd: &Command) -> &'static str {
    match cmd {
        Command::Check { .. } => "CHECK",
        Command::Uncheck { .. } => "UNCHECK",
        Command::Add { .. } => "ADD",
        Command::Delete { .. } => "DELETE",
        Command::Chain { .. } => "CHAIN",
        _ => cmd_name_ext(cmd),
    }
}

fn cmd_name_ext(cmd: &Command) -> &'static str {
    match cmd {
        Command::AddSection { .. } => "ADD_SECTION",
        Command::AddSubsection { .. } => "ADD_SUBSECTION",
        Command::Update { .. } => "UPDATE",
        Command::Note { .. } => "NOTE",
        Command::Move { .. } => "MOVE",
        Command::ReplaceSection { .. } => "SECTION_REPLACE",
        _ => "UNKNOWN",
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

fn parse_command_line(line: &str) -> Result<Command, String> {
    let (cmd, args) = split_cmd(line).ok_or_else(|| "Empty command".to_string())?;
    parse_by_type(cmd, args)
}

fn parse_by_type(cmd: &str, args: &str) -> Result<Command, String> {
    match cmd {
        "CHECK" | "UNCHECK" | "DELETE" => parse_basic(cmd, args),
        "ADD" | "UPDATE" | "NOTE" => parse_content(cmd, args),
        _ => parse_struct(cmd, args),
    }
}

fn parse_basic(cmd: &str, args: &str) -> Result<Command, String> {
    let path = args.trim();
    if path.is_empty() { return Err("Requires task path".into()); }
    match cmd {
        "CHECK" => Ok(Command::Check { path: path.into() }),
        "UNCHECK" => Ok(Command::Uncheck { path: path.into() }),
        "DELETE" => Ok(Command::Delete { path: path.into() }),
        _ => Err(format!("Unknown: {cmd}")),
    }
}

fn parse_content(cmd: &str, args: &str) -> Result<Command, String> {
    match cmd {
        "ADD" => parse_add(args),
        "UPDATE" => parse_update(args),
        "NOTE" => parse_note(args),
        _ => Err(format!("Unknown: {cmd}")),
    }
}

fn parse_struct(cmd: &str, args: &str) -> Result<Command, String> {
    match cmd {
        "MOVE" => parse_move(args),
        "SECTION" => parse_add_section(args),
        "SUBSECTION" => parse_subsection(args),
        "CHAIN" => parse_chain(args),
        "REPLACE_SECTION" => parse_replace_section(args),
        _ => Err(format!("Unknown command: {cmd}")),
    }
}

fn split_cmd(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.splitn(2, ' ');
    let cmd = parts.next()?;
    if cmd.is_empty() { return None; }
    Some((cmd, parts.next().unwrap_or("")))
}

fn parse_add(args: &str) -> Result<Command, String> {
    let (parent, rest) = str_utils::split_first_word(args);
    if parent.is_empty() { return Err("ADD needs parent".into()); }
    let (text, after) = str_utils::parse_quoted_with_after(rest)?;
    Ok(Command::Add { parent: parent.into(), text, after })
}

fn parse_update(args: &str) -> Result<Command, String> {
    let (path, rest) = str_utils::split_first_word(args);
    if path.is_empty() { return Err("UPDATE needs path".into()); }
    Ok(Command::Update { path: path.into(), text: str_utils::parse_quoted(rest)? })
}

fn parse_note(args: &str) -> Result<Command, String> {
    let (path, rest) = str_utils::split_first_word(args);
    if path.is_empty() { return Err("NOTE needs path".into()); }
    Ok(Command::Note { path: path.into(), note: str_utils::parse_quoted(rest)? })
}

fn parse_move(args: &str) -> Result<Command, String> {
    let parts: Vec<&str> = args.split_whitespace().collect();
    if parts.len() < 3 { return Err("MOVE: path AFTER|BEFORE|TO target".into()); }
    let pos = parse_move_position(parts[1], parts[2])?;
    Ok(Command::Move { path: parts[0].into(), position: pos })
}

fn parse_move_position(keyword: &str, target: &str) -> Result<MovePosition, String> {
    match keyword.to_uppercase().as_str() {
        "AFTER" => Ok(MovePosition::After(target.into())),
        "BEFORE" => Ok(MovePosition::Before(target.into())),
        "TO" => Ok(MovePosition::EndOfSection(target.into())),
        _ => Err("Invalid position (use AFTER, BEFORE, or TO)".into()),
    }
}

fn parse_add_section(args: &str) -> Result<Command, String> {
    let heading = str_utils::parse_quoted(args).unwrap_or_else(|_| args.trim().to_string());
    if heading.is_empty() { return Err("SECTION needs heading".into()); }
    Ok(Command::AddSection { heading })
}

fn parse_subsection(args: &str) -> Result<Command, String> {
    let (parent, rest) = str_utils::split_first_word(args);
    if parent.is_empty() { return Err("SUBSECTION needs parent".into()); }
    let heading = str_utils::parse_quoted(rest).unwrap_or_else(|_| rest.trim().to_string());
    if heading.is_empty() { return Err("SUBSECTION needs heading".into()); }
    Ok(Command::AddSubsection { parent: parent.into(), heading })
}

fn parse_replace_section(args: &str) -> Result<Command, String> {
    let id = args.trim();
    if id.is_empty() { return Err("REPLACE_SECTION needs ID".into()); }
    Ok(Command::ReplaceSection { id: id.into(), content: String::new() })
}

fn parse_chain(args: &str) -> Result<Command, String> {
    let (parent, rest) = str_utils::split_first_word(args);
    if parent.is_empty() { return Err("CHAIN needs parent section".into()); }
    let items: Vec<String> = str_utils::parse_quoted_list(rest)?;
    if items.is_empty() { return Err("CHAIN needs at least one item".into()); }
    Ok(Command::Chain { parent: parent.into(), items })
}