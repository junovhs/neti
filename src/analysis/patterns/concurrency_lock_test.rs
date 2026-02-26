// src/analysis/patterns/concurrency_lock_test.rs

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
        assert!(
            v.message.contains("HoL") || v.message.contains("Async"),
            "Async mutex should get HoL message, got: {}",
            v.message
        );
        assert_eq!(v.confidence, Confidence::Medium);
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
