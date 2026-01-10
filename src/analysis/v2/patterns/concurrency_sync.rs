// src/analysis/v2/patterns/concurrency_sync.rs
//! C04: Undocumented synchronization primitives

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

/// C04: Arc<Mutex<T>> without documentation
#[must_use]
pub fn detect_c04(source: &str, root: Node) -> Vec<Violation> {
    let mut violations = Vec::new();
    detect_sync_fields(source, root, &mut violations);
    violations
}

fn detect_sync_fields(source: &str, root: Node, out: &mut Vec<Violation>) {
    let query_str = r"(field_declaration name: (field_identifier) @name) @field";
    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else {
        return;
    };
    
    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(v) = check_sync_field(source, &m) {
            out.push(v);
        }
    }
}

fn check_sync_field(source: &str, m: &tree_sitter::QueryMatch) -> Option<Violation> {
    let field_cap = m.captures.iter().find(|c| c.index == 1)?;
    let field_text = field_cap.node.utf8_text(source.as_bytes()).ok()?;
    
    if !has_sync_type(field_text) {
        return None;
    }
    
    if has_doc_comment(source, field_cap.node) {
        return None;
    }
    
    let name = get_field_name(source, field_cap.node)?;
    Some(build_c04_violation(field_cap.node.start_position().row, &name))
}

fn has_sync_type(text: &str) -> bool {
    text.contains("Arc<Mutex") || text.contains("Arc<RwLock")
}

fn has_doc_comment(source: &str, node: Node) -> bool {
    let row = node.start_position().row;
    if row == 0 {
        return false;
    }
    let lines: Vec<&str> = source.lines().collect();
    lines.get(row - 1)
        .is_some_and(|l| l.trim().starts_with("///") || l.trim().starts_with("//"))
}

fn get_field_name(source: &str, field_node: Node) -> Option<String> {
    for child in field_node.children(&mut field_node.walk()) {
        if child.kind() == "field_identifier" {
            return child.utf8_text(source.as_bytes()).ok().map(String::from);
        }
    }
    None
}

fn build_c04_violation(row: usize, name: &str) -> Violation {
    Violation::with_details(
        row,
        format!("Undocumented sync field `{name}`"),
        "C04",
        ViolationDetails {
            function_name: None,
            analysis: vec![
                "Struct field with Arc<Mutex<T>> indicates shared state.".into(),
                "Document the synchronization contract.".into(),
            ],
            suggestion: Some("Add a `///` doc comment explaining what the lock protects.".into()),
        },
    )
}