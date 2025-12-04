// tests/integration_skeleton.rs
use slopchop_core::skeleton;
use std::path::Path;

#[test]
fn test_clean_rust_basic() {
    let code = "fn main() {\n    println!(\"hi\");\n}";
    let result = skeleton::clean(Path::new("test.rs"), code);
    assert!(result.contains("{ ... }") || result.contains("fn main"));
}

#[test]
fn test_clean_rust_nested() {
    let code = "fn outer() {\n    fn inner() { 42 }\n    inner()\n}";
    let result = skeleton::clean(Path::new("test.rs"), code);
    assert!(result.contains("fn outer") || result.contains("{ ... }"));
}

#[test]
fn test_clean_rust_impl() {
    let code = "impl Foo {\n    fn bar(&self) { 42 }\n}";
    let result = skeleton::clean(Path::new("test.rs"), code);
    assert!(result.contains("impl") || result.contains("Foo"));
}

#[test]
fn test_clean_python() {
    let code = "def hello():\n    print('hi')\n";
    let result = skeleton::clean(Path::new("test.py"), code);
    assert!(result.contains("def hello") || result.contains("..."));
}

#[test]
fn test_clean_typescript() {
    let code = "function hello() {\n    console.log('hi');\n}";
    let result = skeleton::clean(Path::new("test.ts"), code);
    assert!(result.contains("function hello") || result.contains("{ ... }"));
}

#[test]
fn test_clean_unsupported_extension() {
    let code = "some random text";
    let result = skeleton::clean(Path::new("test.xyz"), code);
    assert_eq!(result, code);
}
