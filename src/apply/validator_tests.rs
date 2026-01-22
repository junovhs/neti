use super::*;

#[test]
fn test_path_validation_logic() {
    let cases = vec![
        ("valid/path.rs", true, "Simple relative"),
        ("valid/nested/path.rs", true, "Nested relative"),
        ("/absolute/unix", false, "Unix absolute"),
        ("C:\\absolute\\win", false, "Windows drive absolute"),
        ("\\\\unc\\path", false, "Windows UNC"),
        ("../traversal", false, "Parent traversal"),
        ("path/../traversal", false, "Nested traversal"),
        (".git/config", false, "Blocked dir (.git)"),
        (".env", false, "Blocked file (.env)"),
        ("id_rsa", false, "Blocked sensitive file"),
        ("src/.hidden", false, "Hidden file (not allowed dotfile)"),
        (".gitignore", true, "Allowed dotfile"),
        ("src/slopchop.toml", true, "Allowed deep file"),
        ("foo\0bar", false, "Null byte"),
    ];

    for (path, expected, desc) in cases {
        let result = validate_path(path);
        if expected {
            assert!(result.is_ok(), "Should pass: {desc} ({path})");
        } else {
            assert!(result.is_err(), "Should fail: {desc} ({path})");
        }
    }
}

#[test]
fn test_protected_files_logic() {
    let cases = vec![
        ("slopchop.toml", true),
        ("Cargo.lock", true),
        ("package-lock.json", true),
        ("build.rs", true),
        ("src/main.rs", false),
        ("random.txt", false),
        ("SLOPCHOP.TOML", true), // Case insensitive
    ];

    for (path, protected) in cases {
        assert_eq!(is_protected(path), protected, "Protected check for {path}");
    }
}

#[test]
fn test_content_validation_logic() {
    let cases = vec![
        // (path, content, valid code?, expected valid?)
        ("f.rs", "", false, false), // Empty
        ("f.rs", "   ", false, false), // Whitespace only
        ("f.rs", "fn main() {}", true, true), // Valid Rust
        ("f.rs", "fn main() {", false, false), // Invalid syntax (missing brace)
        ("f.txt", "just text", true, true), // Text file (no syntax check)
        ("f.rs", "```rust\nfn main() {}\n```", true, false), // Markdown fences in code
        ("README.md", "```rust\nfn main() {}\n```", true, true), // Fences allowed in MD
        ("f.rs", "fn f() {\n// ...\n}", true, false), // Truncation detected
        ("f.rs", "fn f() {\n// slopchop:ignore ...\n}", true, true), // Ignored truncation
    ];

    for (path, content, _, expected) in cases {
        let result = validate_content(path, content);
        if expected {
            assert!(result.is_ok(), "Content should be valid for {path}: {content:?}");
        } else {
            assert!(result.is_err(), "Content should be invalid for {path}: {content:?}");
        }
    }
}

#[test]
fn test_is_absolute_os_logic() {
    let cases = vec![
        ("/", true),
        ("/usr/bin", true),
        ("relative/path", false),
        ("C:", true),
        ("D:\\path", true),
        ("\\\\server\\share", true),
        ("plain", false),
    ];

    for (path, expected) in cases {
        assert_eq!(is_absolute_os(path), expected, "is_absolute_os({path})");
    }
}
