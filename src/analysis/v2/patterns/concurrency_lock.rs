// src/analysis/v2/patterns/concurrency_lock.rs
//! C03: `MutexGuard` held across `.await`

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

/// C03: `MutexGuard` held across `.await`
#[must_use]
pub fn detect_c03(source: &str, root: Node) -> Vec<Violation> {
    let mut violations = Vec::new();
    let query_str = r"(function_item (function_modifiers) @mods body: (block) @body) @fn";
    
    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else {
        return violations;
    };
    
    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(v) = check_async_fn(source, &m) {
            violations.push(v);
        }
    }
    violations
}

fn check_async_fn(source: &str, m: &tree_sitter::QueryMatch) -> Option<Violation> {
    let mods_cap = m.captures.iter().find(|c| c.index == 0)?;
    let body_cap = m.captures.iter().find(|c| c.index == 1)?;
    let fn_cap = m.captures.iter().find(|c| c.index == 2)?;
    
    let mods = mods_cap.node.utf8_text(source.as_bytes()).ok()?;
    if !mods.contains("async") {
        return None;
    }
    
    let body_text = body_cap.node.utf8_text(source.as_bytes()).ok()?;
    if !has_lock_and_await(body_text) {
        return None;
    }
    
    if !lock_spans_await(body_text) {
        return None;
    }
    
    let fn_name = get_fn_name(source, fn_cap.node)?;
    Some(build_c03_violation(fn_cap.node.start_position().row, &fn_name))
}

fn has_lock_and_await(body: &str) -> bool {
    let has_lock = body.contains(".lock()") || body.contains(".read()") || body.contains(".write()");
    let has_await = body.contains(".await");
    has_lock && has_await
}

fn lock_spans_await(body: &str) -> bool {
    let mut lock_active = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed == "{" || trimmed == "}" {
            lock_active = false;
            continue;
        }
        if is_lock_line(trimmed) {
            lock_active = true;
        }
        if trimmed.contains("drop(") {
            lock_active = false;
        }
        if lock_active && trimmed.contains(".await") {
            return true;
        }
    }
    false
}

fn is_lock_line(line: &str) -> bool {
    line.starts_with("let ") && (line.contains(".lock()") || line.contains(".read()") || line.contains(".write()"))
}

fn build_c03_violation(row: usize, fn_name: &str) -> Violation {
    Violation::with_details(
        row,
        format!("MutexGuard may be held across .await in `{fn_name}`"),
        "C03",
        ViolationDetails {
            function_name: Some(fn_name.to_string()),
            analysis: vec![
                "Holding MutexGuard across .await can cause deadlocks.".into(),
                "std::sync::Mutex is not async-aware.".into(),
            ],
            suggestion: Some("Use tokio::sync::Mutex or drop guard before await.".into()),
        },
    )
}

fn get_fn_name(source: &str, fn_node: Node) -> Option<String> {
    fn_node.child_by_field_name("name")?
        .utf8_text(source.as_bytes())
        .ok()
        .map(String::from)
}