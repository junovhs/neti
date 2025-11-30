// tests/integration_core.rs
//! Integration tests for the 3 Laws enforcement.
//!
//! VERIFICATION STRATEGY:
//! 1. Isolation: Each syntactic construct (if, match, loop) is tested separately.
//! 2. Boundaries: Tests verify behavior exactly at the limit and limit + 1.
//! 3. Safety: Tests verify that safe alternatives do not trigger violations.

use anyhow::Result;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;
use warden_core::analysis::RuleEngine;
use warden_core::config::{Config, RuleConfig};
use warden_core::types::Violation;

// --- Helpers ---

fn config_complexity(limit: usize) -> RuleConfig {
    RuleConfig {
        max_cyclomatic_complexity: limit,
        ..Default::default()
    }
}

fn config_depth(limit: usize) -> RuleConfig {
    RuleConfig {
        max_nesting_depth: limit,
        ..Default::default()
    }
}

fn config_arity(limit: usize) -> RuleConfig {
    RuleConfig {
        max_function_args: limit,
        ..Default::default()
    }
}

fn config_tokens(limit: usize) -> RuleConfig {
    RuleConfig {
        max_file_tokens: limit,
        ..Default::default()
    }
}

fn scan(content: &str, rules: RuleConfig) -> Result<Vec<Violation>> {
    let dir = TempDir::new()?;
    let file_path = dir.path().join("test.rs");
    let mut file = File::create(&file_path)?;
    write!(file, "{content}")?;

    let mut config = Config::new();
    config.rules = rules;

    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![file_path]);

    Ok(report
        .files
        .into_iter()
        .flat_map(|f| f.violations)
        .collect())
}

// --- Law of Atomicity ---

#[test]
fn test_atomicity_clean_file_passes() -> Result<()> {
    let content = r#"fn main() { println!("Small file"); }"#;
    // Limit is 100, content is ~10 tokens
    let violations = scan(content, config_tokens(100))?;
    assert!(violations.is_empty());
    Ok(())
}

#[test]
fn test_atomicity_large_file_fails() -> Result<()> {
    // Generate content definitely larger than limit
    let content = "fn main() { let x = 1; } ".repeat(20);
    // Limit is 10 tokens
    let violations = scan(&content, config_tokens(10))?;

    assert!(!violations.is_empty());
    assert!(violations[0].message.contains("File size"));
    Ok(())
}

// --- Law of Complexity: Granular Verification ---

#[test]
fn test_complexity_boundary_check() -> Result<()> {
    // Base complexity of a function is 1.
    // Adding one 'if' adds 1.
    // Total = 2.
    let content = "fn f() { if true {} }";

    // Case 1: Limit = 2 (Should Pass)
    let violations = scan(content, config_complexity(2))?;
    assert!(
        violations.is_empty(),
        "Complexity 2 should pass limit 2"
    );

    // Case 2: Limit = 1 (Should Fail)
    let violations = scan(content, config_complexity(1))?;
    assert!(
        violations.iter().any(|v| v.message.contains("Score is 2")),
        "Complexity 2 should fail limit 1"
    );
    Ok(())
}

#[test]
fn test_complexity_construct_match() -> Result<()> {
    // Matches count as branches.
    // Base(1) + Arm(1) + Arm(1) = 3
    let content = r"
        fn f(x: i32) {
            match x {
                1 => {},
                2 => {},
                _ => {}
            }
        }
    ";
    let violations = scan(content, config_complexity(2))?;
    assert!(
        violations.iter().any(|v| v.message.contains("High Complexity")),
        "Match arms must increment complexity"
    );
    Ok(())
}

#[test]
fn test_complexity_construct_loops() -> Result<()> {
    // Loops count as branches.
    // Base(1) + For(1) + While(1) = 3
    let content = r"
        fn f() {
            for _ in 0..10 {}
            while true {}
        }
    ";
    let violations = scan(content, config_complexity(2))?;
    assert!(
        violations.iter().any(|v| v.message.contains("High Complexity")),
        "Loops must increment complexity"
    );
    Ok(())
}

#[test]
fn test_complexity_construct_logic_ops() -> Result<()> {
    // Boolean operators count as branches (short-circuiting).
    // Base(1) + &&(1) + ||(1) = 3
    let content = "fn f(a: bool, b: bool, c: bool) { if a && b || c {} }";
    
    // Note: The 'if' itself counts (1), plus && (1), plus || (1).
    // Total for this function: Base(1) + If(1) + &&(1) + ||(1) = 4.
    
    let violations = scan(content, config_complexity(3))?;
    assert!(
        violations.iter().any(|v| v.message.contains("High Complexity")),
        "Logic operators (&&, ||) must increment complexity"
    );
    Ok(())
}

// --- Law of Complexity: Nesting ---

#[test]
fn test_nesting_boundary() -> Result<()> {
    // Depth: Function block (0) -> If (1).
    let content = "fn f() { if true {} }";

    // Limit 1: Pass
    assert!(scan(content, config_depth(1))?.is_empty());

    // Limit 0: Fail
    // The engine treats function body as depth 0, first block as 1.
    // Let's verify depth 2 fails limit 1.
    let deep = "fn f() { if true { if true {} } }"; // Depth 2
    
    assert!(scan(deep, config_depth(2))?.is_empty());
    
    let violations = scan(deep, config_depth(1))?;
    assert!(
        violations.iter().any(|v| v.message.contains("Deep Nesting")),
        "Depth 2 should fail limit 1"
    );
    Ok(())
}

// --- Law of Complexity: Arity ---

#[test]
fn test_arity_boundary() -> Result<()> {
    let content = "fn f(a: i32, b: i32) {}";

    // Limit 2: Pass
    assert!(scan(content, config_arity(2))?.is_empty());

    // Limit 1: Fail
    let violations = scan(content, config_arity(1))?;
    assert!(
        violations.iter().any(|v| v.message.contains("High Arity")),
        "2 Args should fail limit 1"
    );
    Ok(())
}

// --- Law of Paranoia ---

#[test]
fn test_paranoia_unwrap_fails() -> Result<()> {
    let content = "fn risky() { let x = Some(1); x.unwrap(); }";
    let violations = scan(content, RuleConfig::default())?;

    assert!(violations
        .iter()
        .any(|v| v.message.contains("Banned: '.unwrap()'")));
    Ok(())
}

#[test]
fn test_paranoia_expect_fails() -> Result<()> {
    let content = r#"fn risky() { let x = Some(1); x.expect("boom"); }"#;
    let violations = scan(content, RuleConfig::default())?;

    assert!(violations
        .iter()
        .any(|v| v.message.contains("Banned: '.expect()'")));
    Ok(())
}

#[test]
fn test_paranoia_safe_alternatives_pass() -> Result<()> {
    // Ensure we don't flag valid alternatives or other methods
    // We use r#...# to allow quotes inside the string.
    let content = r#"
        fn safe() { 
            let x = Some(1); 
            x.unwrap_or(0);
            x.unwrap_or_else(|| 0);
            // Result operator should be fine
            let _ = File::open("foo")?; 
        }
    "#;
    let violations = scan(content, RuleConfig::default())?;
    assert!(violations.is_empty(), "Safe error handling should not trigger violations");
    Ok(())
}

// --- Ignore Mechanics ---

#[test]
fn test_warden_ignore_skips_file() -> Result<()> {
    let content = r"
        // warden:ignore
        fn extremely_complex_and_bad(a:i32,b:i32,c:i32,d:i32,e:i32) {
             if true { if true { if true { x.unwrap(); } } }
        }
    ";
    let violations = scan(content, config_complexity(1))?;
    assert!(violations.is_empty(), "warden:ignore should bypass all checks");
    Ok(())
}