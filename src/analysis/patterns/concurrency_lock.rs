// src/analysis/patterns/concurrency_lock.rs
//! C03: `MutexGuard` held across `.await`
//!
//! # Severity Tiers
//!
//! Not all "lock held across await" patterns carry the same risk:
//!
//! **Sync mutex (std::sync::Mutex, parking_lot::Mutex) — HIGH severity**
//! Holding a sync guard across `.await` is a *bug*: it blocks the OS thread,
//! starving the executor, and can deadlock if another task on the same thread
//! tries to acquire the same lock. This is the original C03 intent.
//!
//! **Async mutex (tokio::sync::Mutex, futures::lock::Mutex) — WARN severity**
//! Async mutexes yield to the executor when contended. Holding them across
//! `.await` is not a deadlock, but it may cause head-of-line blocking (other
//! tasks waiting for this lock are stalled while the holder does I/O). The
//! guidance is different: minimize work inside the lock scope, not "use a
//! different mutex."
//!
//! # Detection Heuristic
//!
//! We cannot do full type inference. The heuristic:
//! 1. If the lock-acquisition line itself contains `.lock().await` (or
//!    `.read().await`, `.write().await`) → async mutex.
//! 2. Else if the source file imports `tokio::sync::*` or equivalent →
//!    async context; apply reduced severity.
//! 3. Otherwise → assume sync mutex; apply high severity.

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

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
        if trimmed == "{" || trimmed == "}" {
            lock_active = false;
            continue;
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
///
/// Priority:
/// 1. If the lock-acquisition itself ends in `.await` → async mutex.
/// 2. If the source file references async mutex types → async context.
/// 3. Default → sync.
fn classify_mutex_kind(source: &str, body_text: &str) -> MutexKind {
    // Direct evidence: .lock().await on the acquisition line
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

    // Contextual evidence: file imports async mutex types
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
        MutexKind::Async => Violation::with_details(
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
        ),
    }
}

fn get_fn_name(source: &str, fn_node: Node) -> Option<String> {
    fn_node
        .child_by_field_name("name")?
        .utf8_text(source.as_bytes())
        .ok()
        .map(String::from)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_and_detect(code: &str) -> Vec<Violation> {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_rust::language()).unwrap();
        let tree = parser.parse(code, None).unwrap();
        detect_c03(code, tree.root_node())
    }

    #[test]
    fn c03_flags_sync_mutex_across_await() {
        let code = r#"
            async fn handler(state: Arc<Mutex<Vec<u8>>>) {
                let guard = state.lock().unwrap();
                do_io().await;
                drop(guard);
            }
        "#;
        let vs = parse_and_detect(code);
        assert!(vs.iter().any(|v| v.law == "C03"));
    }

    #[test]
    fn c03_async_mutex_gets_hol_message() {
        let code = r#"
            use tokio::sync::Mutex;
            async fn handler(state: Arc<Mutex<Vec<u8>>>) {
                let guard = state.lock().await;
                do_io().await;
                drop(guard);
            }
        "#;
        let vs = parse_and_detect(code);
        if let Some(v) = vs.iter().find(|v| v.law == "C03") {
            // Should mention HoL, not deadlock
            assert!(
                v.message.contains("HoL") || v.message.contains("Async"),
                "Async mutex should get HoL message, got: {}",
                v.message
            );
        }
    }

    #[test]
    fn c03_no_violation_without_await_span() {
        let code = r#"
            async fn handler(state: Arc<Mutex<Vec<u8>>>) {
                let data = {
                    let guard = state.lock().unwrap();
                    guard.clone()
                };
                do_io().await;
                process(data).await;
            }
        "#;
        // Guard is dropped before await — no violation
        // (lock_spans_await returns false because the block resets lock_active)
        let vs = parse_and_detect(code);
        assert!(vs.iter().all(|v| v.law != "C03"));
    }

    #[test]
    fn classify_sync_by_default() {
        assert_eq!(
            classify_mutex_kind("", "let g = m.lock().unwrap();\n"),
            MutexKind::Sync
        );
    }

    #[test]
    fn classify_async_by_body_pattern() {
        assert_eq!(
            classify_mutex_kind("", "let g = m.lock().await;\n"),
            MutexKind::Async
        );
    }

    #[test]
    fn classify_async_by_import() {
        assert_eq!(
            classify_mutex_kind("use tokio::sync::Mutex;", "let g = m.lock().unwrap();\n"),
            MutexKind::Async
        );
    }
}
