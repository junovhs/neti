// src/graph/imports.rs
use crate::lang::Lang;
use std::path::Path;
use tree_sitter::{Language, Parser, Query, QueryCursor};

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

    let Some(lang) = Lang::from_ext(ext) else {
        return Vec::new();
    };

    let grammar = lang.grammar();
    let query = compile_query(&grammar, lang.q_imports());

    run_query(content, &grammar, &query)
}

fn run_query(source: &str, lang: &Language, query: &Query) -> Vec<String> {
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

    matches.flat_map(|m| m.captures).for_each(|capture| {
        if let Ok(text) = capture.node.utf8_text(source.as_bytes()) {
            imports.push(clean_text(text));
        }
    });

    imports
}

fn clean_text(text: &str) -> String {
    // Remove quotes for JS/TS strings
    text.trim_matches(|c| c == '"' || c == '\'' || c == '`')
        .to_string()
}

fn compile_query(lang: &Language, pattern: &str) -> Query {
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
    fn test_extract_imports() {
        let cases = vec![
            (
                "main.rs",
                r"use std::io; use crate::config::Config; mod tests;",
                vec!["std::io", "crate::config::Config", "tests"],
            ),
            (
                "script.py",
                r"import os; from sys import path; import numpy as np",
                vec!["os", "sys", "numpy"],
            ),
            (
                "app.ts",
                r#"import { Foo } from "./components"; const fs = require('fs'); export * from "./utils";"#,
                vec!["./components", "fs", "./utils"],
            ),
            (
                "lib.rs",
                r"pub use std::collections::HashMap; pub use crate::config;",
                vec!["std::collections::HashMap", "crate::config"],
            ),
        ];

        for (filename, code, expected) in cases {
            let imports = extract(Path::new(filename), code);
            expected.iter().for_each(|item| {
                assert!(
                    imports.contains(&item.to_string()),
                    "File {filename} missing import {item}"
                );
            });
        }
    }
}
