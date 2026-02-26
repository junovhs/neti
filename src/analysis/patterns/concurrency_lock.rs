// src/analysis/patterns/concurrency_lock.rs
//! C03: `MutexGuard` held across `.await`
//!
//! # Severity Tiers
//!
//! **Sync mutex (std::sync::Mutex, parking_lot::Mutex) — HIGH confidence**
//! Holding a sync guard across `.await` blocks the OS thread, starving the
//! executor, and can deadlock if another task tries to acquire the same lock.
//!
//! **Async mutex (tokio::sync::Mutex, futures::lock::Mutex) — MEDIUM confidence**
//! Async mutexes yield on contention. Holding them across `.await` causes
//! head-of-line blocking rather than deadlock.

use crate::types::{Confidence, Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

#[cfg(test)]
#[path = "concurrency_lock_test.rs"]
mod tests;

/// Mutex lock kind, inferred from usage/import patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MutexKind {
    /// std::sync / parking_lot — blocking OS thread
    Sync,
    /// tokio::sync / futures — async-aware
    Async,
}

/// C03: Detects `MutexGuard` held across `.await` in async functions.
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
    let kind = classify_mutex_kind(source, body_text);
    Some(build_c03_violation(
        fn_cap.node.start_position().row,
        &fn_name,
        kind,
    ))
}

fn has_lock_and_await(body: &str) -> bool {
    let has_lock =
        body.contains(".lock()") || body.contains(".read()") || body.contains(".write()");
    has_lock && body.contains(".await")
}

fn lock_spans_await(body: &str) -> bool {
    let mut lock_active = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed == "{" || trimmed.starts_with('}') {
            lock_active = false;
        }
        if is_lock_assignment(trimmed) {
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

fn is_lock_assignment(line: &str) -> bool {
    line.starts_with("let ")
        && (line.contains(".lock()") || line.contains(".read()") || line.contains(".write()"))
}

/// Infers whether the mutex in use is sync or async.
fn classify_mutex_kind(source: &str, body_text: &str) -> MutexKind {
    for line in body_text.lines() {
        let trimmed = line.trim();
        if is_lock_assignment(trimmed)
            && (trimmed.contains(".lock().await")
                || trimmed.contains(".read().await")
                || trimmed.contains(".write().await"))
        {
            return MutexKind::Async;
        }
    }

    if has_async_mutex_import(source) {
        return MutexKind::Async;
    }

    MutexKind::Sync
}

/// Returns `true` if the source file imports an async-aware mutex type.
fn has_async_mutex_import(source: &str) -> bool {
    source.contains("tokio::sync::Mutex")
        || source.contains("tokio::sync::RwLock")
        || source.contains("tokio::sync::Semaphore")
        || source.contains("futures::lock::Mutex")
        || source.contains("futures_util::lock::Mutex")
        || source.contains("async_std::sync::Mutex")
        || source.contains("async_lock::Mutex")
}

fn build_c03_violation(row: usize, fn_name: &str, kind: MutexKind) -> Violation {
    match kind {
        MutexKind::Sync => Violation::with_details(
            row,
            format!("Sync MutexGuard may be held across `.await` in `{fn_name}`"),
            "C03",
            ViolationDetails {
                function_name: Some(fn_name.to_string()),
                analysis: vec![
                    "Holding a sync guard across .await blocks the OS thread.".into(),
                    "std::sync::Mutex is not async-aware — deadlock risk.".into(),
                ],
                suggestion: Some(
                    "Use `tokio::sync::Mutex` or drop the guard before the await point.".into(),
                ),
            },
        ),
        MutexKind::Async => {
            let mut v = Violation::with_details(
                row,
                format!("Async MutexGuard held across `.await` in `{fn_name}` — HoL risk"),
                "C03",
                ViolationDetails {
                    function_name: Some(fn_name.to_string()),
                    analysis: vec![
                        "Async mutexes yield on contention, but holding them across I/O".into(),
                        "stalls other tasks waiting for this lock (head-of-line blocking).".into(),
                    ],
                    suggestion: Some(
                        "Minimize work inside the lock scope; release before awaiting I/O.".into(),
                    ),
                },
            );
            v.confidence = Confidence::Medium;
            v.confidence_reason =
                Some("async mutex — not a deadlock, but may cause head-of-line blocking".into());
            v
        }
    }
}

fn get_fn_name(source: &str, fn_node: Node) -> Option<String> {
    fn_node
        .child_by_field_name("name")?
        .utf8_text(source.as_bytes())
        .ok()
        .map(String::from)
}
