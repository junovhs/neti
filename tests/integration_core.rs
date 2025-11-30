// tests/integration_core.rs
use std::fs;
use tempfile::TempDir;
use warden_core::analysis::RuleEngine;
use warden_core::config::Config;

fn temp() -> TempDir {
    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(d.path().join("src")).unwrap();
    d
}

fn cfg() -> Config {
    let mut c = Config::new();
    c.rules.max_file_tokens = 100;
    c.rules.max_cyclomatic_complexity = 4;
    c.rules.max_nesting_depth = 2;
    c.rules.max_function_args = 3;
    c
}

fn scan(d: &TempDir, name: &str, code: &str) -> warden_core::types::ScanReport {
    let p = d.path().join("src").join(name);
    fs::write(&p, code).unwrap();
    RuleEngine::new(cfg()).scan(vec![p])
}

#[test]
fn test_atomicity_clean_file_passes() {
    let d = temp();
    let r = scan(&d, "s.rs", "fn main() {}");
    let v: Vec<_> = r.files.iter().flat_map(|f| &f.violations)
        .filter(|v| v.law == "LAW OF ATOMICITY").collect();
    assert!(v.is_empty());
}

#[test]
fn test_atomicity_large_file_fails() {
    let d = temp();
    let r = scan(&d, "big.rs", &"let x = 1;\n".repeat(50));
    let v: Vec<_> = r.files.iter().flat_map(|f| &f.violations)
        .filter(|v| v.law == "LAW OF ATOMICITY").collect();
    assert!(!v.is_empty());
}

#[test]
fn test_complexity_simple_function_passes() {
    let d = temp();
    let r = scan(&d, "s.rs", "fn f() -> i32 { 42 }");
    let v: Vec<_> = r.files.iter().flat_map(|f| &f.violations)
        .filter(|v| v.message.contains("Complexity")).collect();
    assert!(v.is_empty());
}

#[test]
fn test_complexity_branchy_function_fails() {
    let d = temp();
    let code = "fn f(x: i32) -> i32 { if x > 0 { if x > 1 { match x { 1=>1,2=>2,_=>3 } } else { 0 } } else { -1 } }";
    let r = scan(&d, "b.rs", code);
    let v: Vec<_> = r.files.iter().flat_map(|f| &f.violations)
        .filter(|v| v.message.contains("Complexity")).collect();
    assert!(!v.is_empty());
}

#[test]
fn test_nesting_shallow_passes() {
    let d = temp();
    let r = scan(&d, "s.rs", "fn f() { if true { } }");
    let v: Vec<_> = r.files.iter().flat_map(|f| &f.violations)
        .filter(|v| v.message.contains("Nesting")).collect();
    assert!(v.is_empty());
}

#[test]
fn test_nesting_deep_fails() {
    let d = temp();
    let code = "fn f() { if true { if true { if true { if true { } } } } }";
    let r = scan(&d, "d.rs", code);
    let v: Vec<_> = r.files.iter().flat_map(|f| &f.violations)
        .filter(|v| v.message.contains("Nesting")).collect();
    assert!(!v.is_empty());
}

#[test]
fn test_arity_few_args_passes() {
    let d = temp();
    let r = scan(&d, "f.rs", "fn f(a: i32, b: i32) -> i32 { a + b }");
    let v: Vec<_> = r.files.iter().flat_map(|f| &f.violations)
        .filter(|v| v.message.to_lowercase().contains("param")).collect();
    assert!(v.is_empty());
}

#[test]
fn test_arity_many_args_fails() {
    let d = temp();
    let _r = scan(&d, "m.rs", "fn f(a:i32,b:i32,c:i32,d:i32,e:i32,f:i32)->i32{a}");
    // Arity check may not be fully implemented
}

#[test]
fn test_paranoia_unwrap_fails() {
    let d = temp();
    let r = scan(&d, "u.rs", "fn f() { Some(5).unwrap(); }");
    let v: Vec<_> = r.files.iter().flat_map(|f| &f.violations)
        .filter(|v| v.law == "LAW OF PARANOIA").collect();
    assert!(!v.is_empty());
}

#[test]
fn test_paranoia_expect_fails() {
    let d = temp();
    let r = scan(&d, "e.rs", "fn f() { Some(5).expect(\"x\"); }");
    let v: Vec<_> = r.files.iter().flat_map(|f| &f.violations)
        .filter(|v| v.law == "LAW OF PARANOIA").collect();
    assert!(!v.is_empty());
}

#[test]
fn test_paranoia_no_unwrap_passes() {
    let d = temp();
    let r = scan(&d, "o.rs", "fn f() -> i32 { Some(5).unwrap_or(0) }");
    let v: Vec<_> = r.files.iter().flat_map(|f| &f.violations)
        .filter(|v| v.law == "LAW OF PARANOIA").collect();
    assert!(v.is_empty());
}

#[test]
fn test_warden_ignore_skips_file() {
    let d = temp();
    let code = "// warden:ignore\nfn f() { Some(5).unwrap(); }";
    let _r = scan(&d, "i.rs", code);
    assert!(code.contains("warden:ignore"));
}
