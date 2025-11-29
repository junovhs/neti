use std::path::Path;
use warden_core::skeleton;

#[test]
fn test_clean_rust_basic() {
    let input = r#"
fn main() {
    println!("Hello");
    let x = 1;
}

struct Foo;

impl Foo {
    fn bar(&self) -> bool {
        true
    }
}
"#;

    let expected = r#"
fn main() { ... }

struct Foo;

impl Foo {
    fn bar(&self) -> bool { ... }
}
"#;

    let cleaned = skeleton::clean(Path::new("test.rs"), input);
    assert_eq!(cleaned.trim(), expected.trim());
}

#[test]
fn test_clean_rust_nested() {
    // We expect the OUTERMOST function block to be replaced.
    // Inner functions should disappear with the body.
    let input = r#"
fn outer() {
    fn inner() {
        println!("inner");
    }
    inner();
}
"#;

    let expected = r#"
fn outer() { ... }
"#;

    let cleaned = skeleton::clean(Path::new("test.rs"), input);
    assert_eq!(cleaned.trim(), expected.trim());
}

#[test]
fn test_clean_python() {
    let input = r#"
def hello(name):
    print(f"Hello {name}")
    return True

class Greeter:
    def greet(self):
        pass
"#;

    let expected = r#"
def hello(name): ...

class Greeter:
    def greet(self): ...
"#;

    let cleaned = skeleton::clean(Path::new("test.py"), input);
    assert_eq!(cleaned.trim(), expected.trim());
}

#[test]
fn test_clean_typescript() {
    let input = r#"
function add(a: number, b: number): number {
    return a + b;
}

class Calculator {
    subtract(a: number, b: number) {
        return a - b;
    }
}

const multiply = (a, b) => {
    return a * b;
};
"#;

    // Note: arrow functions with block bodies `{ ... }` are supported.
    // Expression bodies `=> a * b` are NOT replaced by current query, which looks for `statement_block`.
    let expected = r#"
function add(a: number, b: number): number { ... }

class Calculator {
    subtract(a: number, b: number) { ... }
}

const multiply = (a, b) => { ... };
"#;

    let cleaned = skeleton::clean(Path::new("test.ts"), input);
    assert_eq!(cleaned.trim(), expected.trim());
}

#[test]
fn test_clean_unsupported_extension() {
    let input = "some random text";
    let cleaned = skeleton::clean(Path::new("test.txt"), input);
    assert_eq!(cleaned, input);
}