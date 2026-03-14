//! AST Visitor for analysis. Extracts high-level structure (Scopes/Methods).

use super::extract::RustExtractor; // Updated import
use super::scope::Scope;
use crate::lang::Lang;
use std::collections::HashMap;
use tree_sitter::Node;

pub struct AstVisitor<'a> {
    source: &'a str,
    lang: Lang,
}

impl<'a> AstVisitor<'a> {
    #[must_use]
    pub fn new(source: &'a str, lang: Lang) -> Self {
        Self { source, lang }
    }

    /// Extracts all scopes (classes/structs/enums) from the AST.
    #[must_use]
    pub fn extract_scopes(&self, root: Node) -> HashMap<String, Scope> {
        let mut scopes = HashMap::new();
        if self.lang == Lang::Rust {
            RustExtractor::extract_scopes(self.source, root, &mut scopes);
        }
        scopes
    }
}
