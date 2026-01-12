// src/analysis/v2/worker.rs
//! Worker module for file parsing and analysis.
//! Handles IO, Tree-sitter parsing, and visitor extraction.

use super::{patterns, scope, visitor};
use crate::lang::Lang;
use crate::types::Violation;
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::Parser;

/// Results extracted from a single file.
pub struct FileAnalysis {
    pub violations: Vec<Violation>,
    pub scopes: HashMap<String, scope::Scope>,
    pub path_str: String,
}

/// Analyzes a file for patterns and structural scopes.
#[must_use]
pub fn scan_file(path: &Path) -> Option<FileAnalysis> {
    let source = std::fs::read_to_string(path).ok()?;
    
    // 1. Detect Regex/AST Patterns (State, Security, etc.)
    let violations = patterns::detect_all(path, &source);

    // 2. Extract Structural Scopes (Classes, Structs)
    let (scopes, path_str) = extract_scopes(path, &source);

    Some(FileAnalysis {
        violations,
        scopes,
        path_str,
    })
}

fn extract_scopes(path: &Path, source: &str) -> (HashMap<String, scope::Scope>, String) {
    let empty = (HashMap::new(), String::new());

    let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
        return empty;
    };

    let Some(lang) = Lang::from_ext(ext) else {
        return empty;
    };

    // Currently only Rust has full scope extraction
    if lang != Lang::Rust {
        return empty;
    }

    let mut parser = Parser::new();
    if parser.set_language(lang.grammar()).is_err() {
        return empty;
    }

    let Some(tree) = parser.parse(source, None) else {
        return empty;
    };

    let visitor = visitor::AstVisitor::new(source, lang);
    let scopes = visitor.extract_scopes(tree.root_node());
    let path_string = path.to_string_lossy().to_string();

    (scopes, path_string)
}