// src/roadmap/cmd_helpers.rs
//! Low-level line manipulation helpers for roadmap commands.

use crate::roadmap::parser::slugify;
use crate::roadmap::types::Roadmap;

#[must_use]
pub fn find_line_idx(roadmap: &Roadmap, path: &str) -> Option<usize> {
    find_line_idx_in_lines(&roadmap.raw.lines().collect::<Vec<_>>(), path)
}

#[must_use]
pub fn find_line_idx_in_lines(lines: &[&str], path: &str) -> Option<usize> {
    let search = path.split('/').next_back().unwrap_or(path);
    let s_slug = slugify(search);
    lines.iter().position(|l| is_task(l) && slugify(l).contains(&s_slug))
}

#[must_use]
pub fn is_task(line: &str) -> bool {
    line.trim().starts_with("- [")
}

pub fn replace_raw(roadmap: &mut Roadmap, idx: usize, line: String) {
    modify_lines(roadmap, |lines| {
        if idx < lines.len() {
            lines[idx] = line;
        }
    });
}

pub fn insert_raw(roadmap: &mut Roadmap, idx: usize, line: String) {
    modify_lines(roadmap, |lines| {
        if idx <= lines.len() {
            lines.insert(idx, line);
        }
    });
}

pub fn remove_raw(roadmap: &mut Roadmap, idx: usize) {
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

#[must_use]
pub fn scan_insertion_point(lines: &[&str], parent: &str, after: Option<&str>) -> Option<usize> {
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
        if slugify(line).contains(p_slug) {
            state.in_sec = true;
            state.sec_start = Some(i + 1);
        } else if state.in_sec {
            state.in_sec = false;
        }
        return;
    }

    if state.in_sec && is_task(line) {
        state.last_task = Some(i);
        if let Some(tgt) = after {
            if slugify(line).contains(&slugify(tgt)) {
                state.found_index = Some(i + 1);
            }
        }
    }
}