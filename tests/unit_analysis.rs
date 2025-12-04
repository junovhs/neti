// tests/unit_analysis.rs
use slopchop_core::analysis::ast::Analyzer;
use slopchop_core::analysis::RuleEngine;
use slopchop_core::config::{Config, RuleConfig};
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

// --- Helper for AST Analysis ---
fn analyze(lang: &str, code: &str, complexity: usize) -> bool {
    let analyzer = Analyzer::new();
    let config = RuleConfig {
        max_cyclomatic_complexity: complexity,
        max_function_words: 5, // Default for naming tests
        ..Default::default()
    };

    let violations = analyzer.analyze(lang, "test", code, &config);
    !violations.is_empty()
}

// --- Helper for File-Level Ignores (RuleEngine) ---
fn check_ignore(content: &str) -> bool {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.txt"); // Extension doesn't matter for ignores
    let mut file = File::create(&file_path).unwrap();
    write!(file, "{content}").unwrap();

    let config = Config::default();
    let engine = RuleEngine::new(config);

    // If ignored, analyze_file returns None (or empty report? RuleEngine::analyze_file returns Option<FileReport>)
    // Wait, scan() returns ScanReport.files. If analyze_file returns None, it is filtered out.
    // So if files list is empty, it was ignored.

    let report = engine.scan(vec![file_path]);
    report.files.is_empty()
}

#[test]
fn test_js_complexity() {
    // 1 (Func) + 1 (If) + 1 (For) = 3
    let code = "function f() { if(true) { for(;;) {} } }";
    assert!(analyze("js", code, 2), "Should fail limit 2");
    assert!(!analyze("js", code, 3), "Should pass limit 3");
}

#[test]
fn test_python_complexity() {
    // 1 (Def) + 1 (If) + 1 (While) = 3
    let code = "def f():\n  if True:\n    while True:\n      pass";
    assert!(analyze("py", code, 2), "Should fail limit 2");
    assert!(!analyze("py", code, 3), "Should pass limit 3");
}

#[test]
fn test_snake_case_words() {
    let analyzer = Analyzer::new();
    let config = RuleConfig {
        max_function_words: 3,
        ..Default::default()
    };

    // "this_is_too_long" = 4 words
    let code = "fn this_is_too_long() {}";
    let v = analyzer.analyze("rs", "t.rs", code, &config);
    assert!(!v.is_empty(), "Should detect long snake_case");

    // "short_one" = 2 words
    let code = "fn short_one() {}";
    let v = analyzer.analyze("rs", "t.rs", code, &config);
    assert!(v.is_empty(), "Should allow short snake_case");
}

#[test]
fn test_camel_case_words() {
    let analyzer = Analyzer::new();
    let config = RuleConfig {
        max_function_words: 3,
        ..Default::default()
    };

    // "ThisIsTooLong" = 4 words
    let code = "function ThisIsTooLong() {}";
    let v = analyzer.analyze("js", "t.js", code, &config);
    assert!(!v.is_empty(), "Should detect long CamelCase");

    // "ShortOne" = 2 words
    let code = "function ShortOne() {}";
    let v = analyzer.analyze("js", "t.js", code, &config);
    assert!(v.is_empty(), "Should allow short CamelCase");
}

#[test]
fn test_slopchop_ignore_hash() {
    let content = "# slopchop:ignore\nfn extremely_bad_function() { if true { if true { } } }";
    assert!(
        check_ignore(content),
        "Should ignore file with hash comment"
    );
}

#[test]
fn test_slopchop_ignore_html() {
    let content = "<!-- slopchop:ignore -->\nfn bad() {}";
    assert!(
        check_ignore(content),
        "Should ignore file with html comment"
    );
}
