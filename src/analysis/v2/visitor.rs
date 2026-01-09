// src/analysis/v2/visitor.rs
//! AST Visitor for Scan v2.0. Extracts high-level structure (Scopes/Methods).

use super::cognitive::CognitiveAnalyzer;
use super::scope::{Method, Scope};
use crate::lang::Lang;
use std::collections::{HashMap, HashSet};
use tree_sitter::{Node, Query, QueryCursor, TreeCursor};

pub struct AstVisitor<'a> {
    source: &'a str,
    lang: Lang,
}

impl<'a> AstVisitor<'a> {
    #[must_use]
    pub fn new(source: &'a str, lang: Lang) -> Self {
        Self { source, lang }
    }

    /// Extracts all scopes (classes/structs) from the AST.
    #[must_use]
    pub fn extract_scopes(&self, root: Node) -> HashMap<String, Scope> {
        let mut scopes = HashMap::new();
        if self.lang == Lang::Rust {
            self.extract_rust_scopes(root, &mut scopes);
        }
        scopes
    }

    fn extract_rust_scopes(&self, root: Node, out: &mut HashMap<String, Scope>) {
        let q_str = "(struct_item name: (type_identifier) @name)";
        let query = Query::new(self.lang.grammar(), q_str).expect("Valid Rust query");
        let mut cursor = QueryCursor::new();
        
        for m in cursor.matches(&query, root, self.source.as_bytes()) {
            if let Some(cap) = m.captures.first() {
                if let Ok(name) = cap.node.utf8_text(self.source.as_bytes()) {
                    out.insert(name.to_string(), Scope::new(name));
                }
            }
        }
        self.extract_rust_impls(root, out);
    }

    fn extract_rust_impls(&self, root: Node, out: &mut HashMap<String, Scope>) {
        let q_str = "(impl_item type: (type_identifier) @name body: (declaration_list) @body)";
        let query = Query::new(self.lang.grammar(), q_str).expect("Valid Rust query");
        let mut cursor = QueryCursor::new();

        for m in cursor.matches(&query, root, self.source.as_bytes()) {
            self.process_impl_match(&m, out);
        }
    }

    fn process_impl_match(&self, m: &tree_sitter::QueryMatch, out: &mut HashMap<String, Scope>) {
        let mut name = String::new();
        let mut body_node = None;

        for cap in m.captures {
            if cap.index == 0 {
                name = cap.node.utf8_text(self.source.as_bytes()).unwrap_or("").to_string();
            } else if cap.index == 1 {
                body_node = Some(cap.node);
            }
        }

        if let Some(body) = body_node {
            let scope = out.entry(name).or_insert_with(|| Scope::new("Unknown"));
            self.process_impl_body(body, scope);
        }
    }

    fn process_impl_body(&self, body: Node, scope: &mut Scope) {
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            if child.kind() == "function_item" {
                if let Some(method) = self.extract_rust_method(child) {
                    scope.methods.insert(method.name.clone(), method);
                }
            }
        }
    }

    fn extract_rust_method(&self, node: Node) -> Option<Method> {
        let name_node = node.child_by_field_name("name")?;
        let name = name_node.utf8_text(self.source.as_bytes()).ok()?.to_string();

        let mut method = Method {
            name,
            field_access: HashSet::new(),
            internal_calls: HashSet::new(),
            external_calls: HashSet::new(),
            cognitive_complexity: CognitiveAnalyzer::calculate(node, self.source),
        };

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            self.walk_body_recursive(body, &mut cursor, &mut method);
        }

        Some(method)
    }

    fn walk_body_recursive(&self, node: Node, cursor: &mut TreeCursor, method: &mut Method) {
        self.check_node_heuristics(node, method);

        if cursor.goto_first_child() {
            loop {
                self.walk_body_recursive(cursor.node(), cursor, method);
                if !cursor.goto_next_sibling() { break; }
            }
            cursor.goto_parent();
        }
    }

    fn check_node_heuristics(&self, node: Node, method: &mut Method) {
        match node.kind() {
            "field_expression" => {
                if let Some(field) = self.extract_rust_field(node) {
                    method.field_access.insert(field);
                }
            }
            "call_expression" => {
                if let Some(call_target) = self.extract_rust_call(node) {
                    if call_target.starts_with("self.") {
                        method.internal_calls.insert(call_target.replace("self.", ""));
                    } else {
                        method.external_calls.insert(call_target);
                    }
                }
            }
            _ => {}
        }
    }

    fn extract_rust_field(&self, node: Node) -> Option<String> {
        let val = node.child_by_field_name("value")?;
        let field = node.child_by_field_name("field")?;
        if val.utf8_text(self.source.as_bytes()).ok()? == "self" {
            return Some(field.utf8_text(self.source.as_bytes()).ok()?.to_string());
        }
        None
    }

    fn extract_rust_call(&self, node: Node) -> Option<String> {
        let function = node.child_by_field_name("function")?;
        function.utf8_text(self.source.as_bytes()).ok().map(String::from)
    }
}