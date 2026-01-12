// src/analysis/v2/patterns/state.rs
//! State pattern detection: S01, S02, S03

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};
use super::get_capture_node;

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
    let idx_mut = query.capture_index_for_name("mut");
    
    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(cap) = get_capture_node(&m, idx_mut) {
            let row = cap.start_position().row;
            let text = extract_first_line(source, cap);
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
    let idx_vis = query.capture_index_for_name("vis");
    let idx_name = query.capture_index_for_name("name");
    let idx_item = query.capture_index_for_name("item");
    
    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, root, source.as_bytes()) {
        let vis = get_capture_node(&m, idx_vis);
        let name = get_capture_node(&m, idx_name);
        let item = get_capture_node(&m, idx_item);
        
        if let (Some(vis), Some(name), Some(item)) = (vis, name, item) {
             if process_s02_check(source, vis, name, item, out) {
                 // violation added
             }
        }
    }
}

fn process_s02_check(source: &str, vis: Node, name: Node, item: Node, out: &mut Vec<Violation>) -> bool {
    let vis_text = vis.utf8_text(source.as_bytes()).unwrap_or("");
    if !vis_text.contains("pub") {
        return false;
    }
    
    let name_text = name.utf8_text(source.as_bytes()).unwrap_or("");
    let item_text = item.utf8_text(source.as_bytes()).unwrap_or("");
    
    if item_text.contains("static mut") {
        return false; // Already caught by S01
    }
    
    if name_text.chars().all(|c| c.is_uppercase() || c == '_') {
        return false; // Const-like naming
    }
    
    out.push(build_s02_violation(item.start_position().row, name_text));
    true
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
    let idx_item = query.capture_index_for_name("item");
    
    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(item) = get_capture_node(&m, idx_item) {
            if let Some(v) = check_s03_container(source, item) {
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