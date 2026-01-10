// src/analysis/v2/patterns/state.rs
//! State pattern detection: S01, S02, S03

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

/// Detects state-related violations in Rust code.
#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut violations = Vec::new();
    detect_s01(source, root, &mut violations);
    detect_s02(source, root, &mut violations);
    detect_s03(source, root, &mut violations);
    violations
}

/// S01: Global mutable declaration - `static mut`
fn detect_s01(source: &str, root: Node, out: &mut Vec<Violation>) {
    let query_str = r"(static_item (mutable_specifier) @mut) @item";
    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else {
        return;
    };
    
    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(cap) = m.captures.iter().find(|c| c.index == 1) {
            let row = cap.node.start_position().row;
            let text = extract_first_line(source, cap.node);
            out.push(build_s01_violation(row, &text));
        }
    }
}

fn build_s01_violation(row: usize, text: &str) -> Violation {
    Violation::with_details(
        row,
        format!("Global mutable state: `{}`", truncate(text, 50)),
        "S01",
        ViolationDetails {
            function_name: None,
            analysis: vec![
                "`static mut` is unsafe and a source of data races.".into(),
                "Global mutable state makes code unpredictable.".into(),
            ],
            suggestion: Some("Use `AtomicUsize`, `Mutex<T>`, or `OnceCell`.".into()),
        },
    )
}

/// S02: Exported mutable - `pub static` (non-const)
fn detect_s02(source: &str, root: Node, out: &mut Vec<Violation>) {
    let query_str = r"(static_item (visibility_modifier) @vis name: (identifier) @name) @item";
    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else {
        return;
    };
    
    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(v) = process_s02_match(source, &m) {
            out.push(v);
        }
    }
}

fn process_s02_match(source: &str, m: &tree_sitter::QueryMatch) -> Option<Violation> {
    let vis_cap = m.captures.iter().find(|c| c.index == 0)?;
    let name_cap = m.captures.iter().find(|c| c.index == 1)?;
    let item_cap = m.captures.iter().find(|c| c.index == 2)?;
    
    let vis = vis_cap.node.utf8_text(source.as_bytes()).ok()?;
    if !vis.contains("pub") {
        return None;
    }
    
    let name = name_cap.node.utf8_text(source.as_bytes()).ok()?;
    let item_text = item_cap.node.utf8_text(source.as_bytes()).ok()?;
    
    if item_text.contains("static mut") {
        return None; // Already caught by S01
    }
    
    if name.chars().all(|c| c.is_uppercase() || c == '_') {
        return None; // Const-like naming
    }
    
    Some(build_s02_violation(item_cap.node.start_position().row, name))
}

fn build_s02_violation(row: usize, name: &str) -> Violation {
    Violation::with_details(
        row,
        format!("Exported static `{name}` may expose shared state"),
        "S02",
        ViolationDetails {
            function_name: None,
            analysis: vec![
                "Public statics can be accessed from anywhere.".into(),
                "This can lead to implicit coupling.".into(),
            ],
            suggestion: Some("Use a function or make the static private.".into()),
        },
    )
}

/// S03: Suspicious global container - `lazy_static` with Mutex
fn detect_s03(source: &str, root: Node, out: &mut Vec<Violation>) {
    let query_str = r#"(macro_invocation macro: (identifier) @mac (#match? @mac "^lazy_static$")) @item"#;
    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else {
        return;
    };
    
    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(cap) = m.captures.iter().find(|c| c.index == 1) {
            if let Some(v) = check_s03_container(source, cap.node) {
                out.push(v);
            }
        }
    }
}

fn check_s03_container(source: &str, node: Node) -> Option<Violation> {
    let text = node.utf8_text(source.as_bytes()).ok()?;
    let has_container = text.contains("Mutex<Vec")
        || text.contains("Mutex<HashMap")
        || text.contains("RwLock<Vec")
        || text.contains("RwLock<HashMap");
    
    if !has_container {
        return None;
    }
    
    Some(Violation::with_details(
        node.start_position().row,
        "Suspicious global container in lazy_static".to_string(),
        "S03",
        ViolationDetails {
            function_name: None,
            analysis: vec![
                "Global containers accumulate state over lifetime.".into(),
                "This pattern often indicates singleton abuse.".into(),
            ],
            suggestion: Some("Pass data through function parameters.".into()),
        },
    ))
}

fn extract_first_line(source: &str, node: Node) -> String {
    node.utf8_text(source.as_bytes())
        .unwrap_or("")
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}...", &s[..max]) }
}