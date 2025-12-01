use crate::roadmap::types::{Roadmap, RoadmapStats, Section, Task, TaskStatus};
use std::path::Path;

impl Roadmap {
    /// Parse a roadmap from markdown content
    #[must_use]
    pub fn parse(content: &str) -> Self {
        let lines: Vec<&str> = content.lines().collect();
        let mut sections = Vec::new();
        let mut title = "Roadmap".to_string();
        let mut i = 0;

        if let Some(first) = lines.first() {
            if let Some(t) = first.strip_prefix("# ") {
                title = t.trim().to_string();
                i = 1;
            }
        }

        while i < lines.len() {
            if let Some((lvl, txt)) = parse_heading(lines[i]) {
                sections.push(parse_section(&lines, &mut i, lvl, txt));
            } else {
                i += 1;
            }
        }

        Self {
            path: None,
            title,
            sections,
            raw: content.into(),
        }
    }

    /// Parse from a file
    /// # Errors
    /// Returns error on file read fail
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let c = std::fs::read_to_string(path)?;
        let mut r = Self::parse(&c);
        r.path = Some(path.display().to_string());
        Ok(r)
    }

    /// Save back to file
    /// # Errors
    /// Returns error on file write fail
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        std::fs::write(path, &self.raw)
    }

    #[must_use]
    pub fn all_tasks(&self) -> Vec<&Task> {
        let mut out = Vec::new();
        for s in &self.sections {
            collect_tasks(s, &mut out);
        }
        out
    }

    #[must_use]
    pub fn find_task(&self, path: &str) -> Option<&Task> {
        self.all_tasks().into_iter().find(|t| t.path == path)
    }

    #[must_use]
    pub fn stats(&self) -> RoadmapStats {
        let t = self.all_tasks();
        let c = t
            .iter()
            .filter(|x| x.status == TaskStatus::Complete)
            .count();
        RoadmapStats {
            total: t.len(),
            complete: c,
            pending: t.len() - c,
        }
    }
}

fn parse_heading(line: &str) -> Option<(u8, String)> {
    let t = line.trim();
    if !t.starts_with("##") {
        return None;
    }
    let lvl = t.chars().take_while(|&c| c == '#').count();
    if lvl < 2 {
        return None;
    }

    let level = u8::try_from(lvl).ok()?;
    Some((level, t[lvl..].trim().into()))
}

fn parse_section(lines: &[&str], i: &mut usize, lvl: u8, heading: String) -> Section {
    let start = *i;
    let id = crate::roadmap::slugify(&heading);
    let mut tasks = Vec::new();
    let mut subs = Vec::new();
    let mut raw = String::new();

    *i += 1;

    while *i < lines.len() {
        let line = lines[*i];
        if let Some((next_lvl, next_txt)) = parse_heading(line) {
            if next_lvl <= lvl {
                break;
            }
            subs.push(parse_section(lines, i, next_lvl, next_txt));
            continue;
        }

        if let Some(mut task) = parse_task(line, *i) {
            task.path = format!("{id}/{}", task.id);
            tasks.push(task);
        } else {
            raw.push_str(line);
            raw.push('\n');
        }
        *i += 1;
    }

    Section {
        id,
        heading,
        level: lvl,
        theme: None,
        tasks,
        subsections: subs,
        raw_content: raw,
        line_start: start,
        line_end: *i,
    }
}

fn parse_task(line: &str, line_num: usize) -> Option<Task> {
    let t = line.trim();
    if !t.starts_with("- [") {
        return None;
    }

    let (stat, rest) = if let Some(stripped) = t.strip_prefix("- [ ]") {
        (TaskStatus::Pending, stripped)
    } else {
        (TaskStatus::Complete, &t[5..])
    };

    let mut parts = rest.split("<!--");
    let text_part = parts.next()?.trim().trim_matches(|c| c == '*' || c == ' ');
    let id = crate::roadmap::slugify(text_part);

    if id.is_empty() {
        return None;
    }

    let mut tests = Vec::new();
    for part in parts {
        if let Some(content) = part.split("-->").next() {
            if let Some(path) = content.trim().strip_prefix("test:") {
                 tests.push(path.trim().to_string());
            }
        }
    }

    Some(Task {
        id,
        path: String::new(),
        text: text_part.into(),
        status: stat,
        indent: 0,
        line: line_num,
        children: vec![],
        tests,
    })
}

fn collect_tasks<'a>(s: &'a Section, out: &mut Vec<&'a Task>) {
    for t in &s.tasks {
        out.push(t);
    }
    for sub in &s.subsections {
        collect_tasks(sub, out);
    }
}

/// Convert text to a URL-safe slug
#[must_use]
pub fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}