// src/graph/imports.rs
use std::path::Path;
use std::sync::LazyLock;
use tree_sitter::{Language, Parser, Query, QueryCursor};

static EXTRACTOR: LazyLock<ImportExtractor> = LazyLock::new(ImportExtractor::new);

struct ImportExtractor {
    rust: Query,
    python: Query,
    javascript: Query,
}

impl ImportExtractor {
    fn new() -> Self {
        Self {
            rust: compile_query(
                tree_sitter_rust::language(),
                r"
                (use_declaration argument: (_) @import)
                (mod_item name: (identifier) @mod)
                ",
            ),
            python: compile_query(
                tree_sitter_python::language(),
                r"
                (import_statement name: (dotted_name) @import)
                (import_from_statement module_name: (dotted_name) @import)
                ",
            ),
            javascript: compile_query(
                tree_sitter_typescript::language_typescript(),
                r#"
                (import_statement source: (string) @import)
                (export_statement source: (string) @import)
                (call_expression
                  function: (identifier) @func
                  arguments: (arguments (string) @import)
                  (#eq? @func "require"))
                "#,
            ),
        }
    }

    fn get_config<'a>(&'a self, lang: &str) -> Option<(Language, &'a Query)> {
        match lang {
            "rs" => Some((tree_sitter_rust::language(), &self.rust)),
            "py" => Some((tree_sitter_python::language(), &self.python)),
            "js" | "jsx" | "ts" | "tsx" => Some((
                tree_sitter_typescript::language_typescript(),
                &self.javascript,
            )),
            _ => None,
        }
    }
}

/// Extracts raw import strings from the given file content.
///
/// # Arguments
/// * `path` - File path (used for language detection).
/// * `content` - Source code.
///
/// # Returns
/// A list of imported module names/paths (e.g., "`std::io`", "./utils", "react").
#[must_use]
pub fn extract(path: &Path, content: &str) -> Vec<String> {
    let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
        return Vec::new();
    };

    let Some((lang, query)) = EXTRACTOR.get_config(ext) else {
        return Vec::new();
    };

    run_query(content, lang, query)
}

fn run_query(source: &str, lang: Language, query: &Query) -> Vec<String> {
    let mut parser = Parser::new();
    if parser.set_language(lang).is_err() {
        return Vec::new();
    }

    let Some(tree) = parser.parse(source, None) else {
        return Vec::new();
    };

    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(query, tree.root_node(), source.as_bytes());
    let mut imports = Vec::new();

    for m in matches {
        for capture in m.captures {
            if let Ok(text) = capture.node.utf8_text(source.as_bytes()) {
                imports.push(clean_text(text));
            }
        }
    }

    imports
}

fn clean_text(text: &str) -> String {
    // Remove quotes for JS/TS strings
    text.trim_matches(|c| c == '"' || c == '\'' || c == '`')
        .to_string()
}

fn compile_query(lang: Language, pattern: &str) -> Query {
    match Query::new(lang, pattern) {
        Ok(q) => q,
        Err(e) => panic!("Invalid import query: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_rust_imports() {
        let code = r"
            use std::io;
            use crate::config::Config;
            mod tests;
        ";
        let imports = extract(Path::new("main.rs"), code);
        assert!(imports.contains(&"std::io".to_string()));
        assert!(imports.contains(&"crate::config::Config".to_string()));
        assert!(imports.contains(&"tests".to_string()));
    }

    #[test]
    fn test_python_imports() {
        let code = r"
            import os
            from sys import path
            import numpy as np
        ";
        let imports = extract(Path::new("script.py"), code);
        assert!(imports.contains(&"os".to_string()));
        assert!(imports.contains(&"sys".to_string()));
        assert!(imports.contains(&"numpy".to_string()));
    }

    #[test]
    fn test_ts_imports() {
        let code = r#"
            import { Foo } from "./components";
            const fs = require('fs');
            export * from "./utils";
        "#;
        let imports = extract(Path::new("app.ts"), code);
        assert!(imports.contains(&"./components".to_string()));
        assert!(imports.contains(&"fs".to_string()));
        assert!(imports.contains(&"./utils".to_string()));
    }
}