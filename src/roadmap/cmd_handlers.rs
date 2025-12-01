use crate::roadmap::parser::slugify;
use crate::roadmap::types::{ApplyResult, MovePosition, Roadmap, TaskStatus};

pub fn handle_check(roadmap: &mut Roadmap, path: &str) -> ApplyResult {
    set_status(roadmap, path, TaskStatus::Complete)
}

pub fn handle_uncheck(roadmap: &mut Roadmap, path: &str) -> ApplyResult {
    set_status(roadmap, path, TaskStatus::Pending)
}

pub fn handle_add(
    roadmap: &mut Roadmap,
    parent: &str,
    text: &str,
    after: Option<&str>,
) -> ApplyResult {
    let lines: Vec<&str> = roadmap.raw.lines().collect();

    if let Some(idx) = scan_insertion_point(&lines, parent, after) {
        insert_raw(roadmap, idx, format!("- [ ] **{text}**"));
        ApplyResult::Success(format!("Added: {text}"))
    } else {
        ApplyResult::NotFound(format!("Section: {parent}"))
    }
}

pub fn handle_add_section(roadmap: &mut Roadmap, heading: &str) -> ApplyResult {
    let new_section = format!("\n\n## {heading}\n");
    roadmap.raw.push_str(&new_section);
    ApplyResult::Success(format!("Added Section: {heading}"))
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
    if let Some(idx) = find_line_idx(roadmap, path) {
        let line = roadmap.raw.lines().nth(idx).unwrap_or("");
        let indent = &line[..line.len() - line.trim_start().len()];
        let mark = if line.to_uppercase().contains("[X]") {
            "[x]"
        } else {
            "[ ]"
        };

        let suffix = if let Some(pos) = line.find("<!--") {
            &line[pos..]
        } else {
            ""
        };

        let new_line = if suffix.is_empty() {
            format!("{indent}- {mark} **{text}**")
        } else {
            format!("{indent}- {mark} **{text}** {suffix}")
        };

        replace_raw(roadmap, idx, new_line);
        ApplyResult::Success(format!("Updated: {path}"))
    } else {
        ApplyResult::NotFound(path.into())
    }
}

pub fn handle_note(roadmap: &mut Roadmap, path: &str, note: &str) -> ApplyResult {
    if let Some(idx) = find_line_idx(roadmap, path) {
        let line = roadmap.raw.lines().nth(idx).unwrap_or("");
        let indent_len = line.len() - line.trim_start().len();
        let prefix = " ".repeat(indent_len + 2);

        insert_raw(roadmap, idx + 1, format!("{prefix}*{note}*"));
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
        MovePosition::After(target) => find_line_idx_in_lines(&lines, target).map(|i| i + 1),
        MovePosition::Before(target) => find_line_idx_in_lines(&lines, target),
        MovePosition::EndOfSection(section) => scan_insertion_point(&lines, section, None),
    };

    if let Some(idx) = target_idx {
        insert_raw(roadmap, idx, line_content);
        ApplyResult::Success(format!("Moved: {path}"))
    } else {
        ApplyResult::Error("Target not found".into())
    }
}

fn set_status(roadmap: &mut Roadmap, path: &str, status: TaskStatus) -> ApplyResult {
    if let Some(idx) = find_line_idx(roadmap, path) {
        if update_line_status(roadmap, idx, status) {
            return ok_res(status, path);
        }
    }
    ApplyResult::NotFound(path.into())
}

// --- Logic Helpers (Copied from cmd_runner.rs for atomic migration) ---

fn scan_insertion_point(lines: &[&str], parent: &str, after: Option<&str>) -> Option<usize> {
    let p_slug = slugify(parent);
    let mut state = ScanState::default();

    for (i, line) in lines.iter().enumerate() {
        process_line(line, i, &p_slug, after, &mut state);
        if let Some(idx) = state.found_index {
            return Some(idx);
        }
    }
    state.last_task.map(|i| i + 1).or(state.sec_start)
}

#[derive(Default)]
struct ScanState {
    in_sec: bool,
    last_task: Option<usize>,
    sec_start: Option<usize>,
    found_index: Option<usize>,
}

fn process_line(line: &str, i: usize, p_slug: &str, after: Option<&str>, state: &mut ScanState) {
    if line.starts_with("##") {
        if check_section_entry(line, p_slug) {
            state.in_sec = true;
            state.sec_start = Some(i + 1);
        } else if state.in_sec {
            state.in_sec = false;
        }
        return;
    }

    if state.in_sec && is_task(line) {
        state.last_task = Some(i);
        if check_after_match(line, after) {
            state.found_index = Some(i + 1);
        }
    }
}

fn check_section_entry(line: &str, parent_slug: &str) -> bool {
    slugify(line).contains(parent_slug)
}

fn check_after_match(line: &str, after: Option<&str>) -> bool {
    if let Some(tgt) = after {
        slugify(line).contains(&slugify(tgt))
    } else {
        false
    }
}

fn find_line_idx(roadmap: &Roadmap, path: &str) -> Option<usize> {
    find_line_idx_in_lines(&roadmap.raw.lines().collect::<Vec<_>>(), path)
}

fn find_line_idx_in_lines(lines: &[&str], path: &str) -> Option<usize> {
    let search = path.split('/').next_back().unwrap_or(path);
    let s_slug = slugify(search);

    lines
        .iter()
        .position(|l| is_task(l) && slugify(l).contains(&s_slug))
}

fn update_line_status(roadmap: &mut Roadmap, idx: usize, status: TaskStatus) -> bool {
    let lines: Vec<&str> = roadmap.raw.lines().collect();
    if idx >= lines.len() {
        return false;
    }

    let line = lines[idx];
    let new = match status {
        TaskStatus::Complete => line.replace("- [ ]", "- [x]"),
        TaskStatus::Pending => line.replace("- [x]", "- [ ]").replace("- [X]", "- [ ]"),
    };
    replace_raw(roadmap, idx, new);
    true
}

fn replace_raw(roadmap: &mut Roadmap, idx: usize, line: String) {
    modify_lines(roadmap, |lines| {
        if idx < lines.len() {
            lines[idx] = line;
        }
    });
}

fn insert_raw(roadmap: &mut Roadmap, idx: usize, line: String) {
    modify_lines(roadmap, |lines| {
        if idx <= lines.len() {
            lines.insert(idx, line);
        }
    });
}

fn remove_raw(roadmap: &mut Roadmap, idx: usize) {
    modify_lines(roadmap, |lines| {
        if idx < lines.len() {
            lines.remove(idx);
        }
    });
}

fn modify_lines<F>(roadmap: &mut Roadmap, f: F)
where
    F: FnOnce(&mut Vec<String>),
{
    let mut lines: Vec<String> = roadmap.raw.lines().map(ToString::to_string).collect();
    f(&mut lines);
    roadmap.raw = lines.join("\n");
}

fn is_task(line: &str) -> bool {
    line.trim().starts_with("- [")
}

fn ok_res(status: TaskStatus, path: &str) -> ApplyResult {
    let act = if status == TaskStatus::Complete {
        "Checked"
    } else {
        "Unchecked"
    };
    ApplyResult::Success(format!("{act}: {path}"))
}