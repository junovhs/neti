// src/roadmap/cmd_handlers.rs
use std::fmt::Write;

use crate::roadmap::cmd_helpers::{
    find_line_idx, find_line_idx_in_lines, insert_raw, remove_raw, replace_raw,
    scan_insertion_point,
};
use crate::roadmap::parser::slugify;
use crate::roadmap::types::{ApplyResult, MovePosition, Roadmap, TaskStatus};

pub fn handle_check(roadmap: &mut Roadmap, path: &str) -> ApplyResult {
    set_status(roadmap, path, TaskStatus::Complete)
}

pub fn handle_uncheck(roadmap: &mut Roadmap, path: &str) -> ApplyResult {
    set_status(roadmap, path, TaskStatus::Pending)
}

pub fn handle_add(roadmap: &mut Roadmap, parent: &str, text: &str, after: Option<&str>) -> ApplyResult {
    let lines: Vec<&str> = roadmap.raw.lines().collect();
    if let Some(idx) = scan_insertion_point(&lines, parent, after) {
        insert_raw(roadmap, idx, format!("- [ ] **{text}**"));
        ApplyResult::Success(format!("Added: {text}"))
    } else {
        ApplyResult::NotFound(format!("Section: {parent}"))
    }
}

pub fn handle_add_section(roadmap: &mut Roadmap, heading: &str) -> ApplyResult {
    let _ = write!(roadmap.raw, "\n\n## {heading}\n");
    ApplyResult::Success(format!("Added Section: {heading}"))
}

pub fn handle_add_subsection(roadmap: &mut Roadmap, parent: &str, heading: &str) -> ApplyResult {
    let lines: Vec<&str> = roadmap.raw.lines().collect();
    let p_slug = slugify(parent);
    let mut in_section = false;
    let mut insert_idx = None;

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("## ") && slugify(line).contains(&p_slug) {
            in_section = true;
        } else if in_section && line.starts_with("## ") {
            insert_idx = Some(i);
            break;
        }
    }

    let idx = insert_idx.unwrap_or(lines.len());
    insert_raw(roadmap, idx, format!("\n### {heading}\n"));
    ApplyResult::Success(format!("Added Subsection: {heading}"))
}

pub fn handle_delete(roadmap: &mut Roadmap, path: &str) -> ApplyResult {
    if let Some(idx) = find_line_idx(roadmap, path) {
        remove_raw(roadmap, idx);
        ApplyResult::Success(format!("Deleted: {path}"))
    } else {
        ApplyResult::NotFound(path.into())
    }
}

pub fn handle_update(roadmap: &mut Roadmap, path: &str, text: &str) -> ApplyResult {
    let Some(idx) = find_line_idx(roadmap, path) else {
        return ApplyResult::NotFound(path.into());
    };
    let line = roadmap.raw.lines().nth(idx).unwrap_or("");
    let indent = &line[..line.len() - line.trim_start().len()];
    let mark = if line.to_uppercase().contains("[X]") { "[x]" } else { "[ ]" };
    let suffix = line.find("<!--").map_or("", |pos| &line[pos..]);
    let new_line = if suffix.is_empty() {
        format!("{indent}- {mark} **{text}**")
    } else {
        format!("{indent}- {mark} **{text}** {suffix}")
    };
    replace_raw(roadmap, idx, new_line);
    ApplyResult::Success(format!("Updated: {path}"))
}

pub fn handle_note(roadmap: &mut Roadmap, path: &str, note: &str) -> ApplyResult {
    if let Some(idx) = find_line_idx(roadmap, path) {
        let line = roadmap.raw.lines().nth(idx).unwrap_or("");
        let indent_len = line.len() - line.trim_start().len();
        insert_raw(roadmap, idx + 1, format!("{:indent$}*{note}*", "", indent = indent_len + 2));
        ApplyResult::Success(format!("Added note: {path}"))
    } else {
        ApplyResult::NotFound(path.into())
    }
}

pub fn handle_move(roadmap: &mut Roadmap, path: &str, position: &MovePosition) -> ApplyResult {
    let Some(src_idx) = find_line_idx(roadmap, path) else {
        return ApplyResult::NotFound(path.into());
    };
    let line_content = roadmap.raw.lines().nth(src_idx).unwrap_or("").to_string();
    remove_raw(roadmap, src_idx);

    let lines: Vec<&str> = roadmap.raw.lines().collect();
    let target_idx = match position {
        MovePosition::After(t) => find_line_idx_in_lines(&lines, t).map(|i| i + 1),
        MovePosition::Before(t) => find_line_idx_in_lines(&lines, t),
        MovePosition::EndOfSection(s) => scan_insertion_point(&lines, s, None),
    };

    if let Some(idx) = target_idx {
        insert_raw(roadmap, idx, line_content);
        ApplyResult::Success(format!("Moved: {path}"))
    } else {
        ApplyResult::Error("Target not found".into())
    }
}

fn set_status(roadmap: &mut Roadmap, path: &str, status: TaskStatus) -> ApplyResult {
    let Some(idx) = find_line_idx(roadmap, path) else {
        return ApplyResult::NotFound(path.into());
    };
    let lines: Vec<&str> = roadmap.raw.lines().collect();
    let line = lines.get(idx).copied().unwrap_or("");
    let new = match status {
        TaskStatus::Complete => line.replace("- [ ]", "- [x]"),
        TaskStatus::Pending => line.replace("- [x]", "- [ ]").replace("- [X]", "- [ ]"),
    };
    replace_raw(roadmap, idx, new);
    let act = if status == TaskStatus::Complete { "Checked" } else { "Unchecked" };
    ApplyResult::Success(format!("{act}: {path}"))
}