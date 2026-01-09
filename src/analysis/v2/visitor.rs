// src/analysis/v2/visitor.rs
//! AST Visitor for Scan v2.0. Extracts high-level structure (Scopes/Methods).

use super::cognitive::CognitiveAnalyzer;
use super::scope::{FieldInfo, Method, Scope};
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

    /// Extracts all scopes (classes/structs/enums) from the AST.
    #[must_use]
    pub fn extract_scopes(&self, root: Node) -> HashMap<String, Scope> {
        let mut scopes = HashMap::new();
        if self.lang == Lang::Rust {
            self.extract_rust_scopes(root, &mut scopes);
        }
        scopes
    }

    fn extract_rust_scopes(&self, root: Node, out: &mut HashMap<String, Scope>) {
        // Extract struct definitions with fields
        self.extract_rust_structs_and_fields(root, out);

        // Extract enum definitions
        self.extract_rust_type_defs(root, out, "(enum_item name: (type_identifier) @name)", true);

        // Extract impl blocks and attach methods to scopes
        self.extract_rust_impls(root, out);
    }

    fn extract_rust_structs_and_fields(&self, root: Node, out: &mut HashMap<String, Scope>) {
        // Query for all field declarations directly to avoid nested query fragility
        let query_str = "(field_declaration) @field";
        let Ok(query) = Query::new(self.lang.grammar(), query_str) else {
            return;
        };
        let mut cursor = QueryCursor::new();

        for m in cursor.matches(&query, root, self.source.as_bytes()) {
            for cap in m.captures {
                let field_node = cap.node;
                self.process_field_node(field_node, out);
            }
        }
    }

    fn process_field_node(&self, field_node: Node, out: &mut HashMap<String, Scope>) {
        // Traverse up to find struct definition
        // field_declaration -> field_declaration_list -> struct_item
        let Some(field_list) = field_node.parent() else { return };
        let Some(struct_node) = field_list.parent() else { return };

        if struct_node.kind() != "struct_item" {
            return;
        }

        // Extract struct name
        let Some(name_node) = struct_node.child_by_field_name("name") else { return };
        let Ok(struct_name) = name_node.utf8_text(self.source.as_bytes()) else { return };
        let struct_row = struct_node.start_position().row + 1;

        // Get or create scope
        let scope = out
            .entry(struct_name.to_string())
            .or_insert_with(|| Scope::new(struct_name, struct_row));

        // Attempt to extract derives if not already present
        // (This is idempotent-ish since we use a Set, but avoiding re-parsing is good)
        if scope.derives.is_empty() {
            self.extract_derives(struct_node, &mut scope.derives);
        }

        // Extract field info
        let Some(field_name_node) = field_node.child_by_field_name("name") else { return };
        let Ok(field_name) = field_name_node.utf8_text(self.source.as_bytes()) else { return };
        
        // Check visibility
        let mut is_public = false;
        // Visibility is an optional child of field_declaration
        for child in field_node.children(&mut field_node.walk()) {
            if child.kind() == "visibility_modifier" {
                if let Ok(vis_text) = child.utf8_text(self.source.as_bytes()) {
                    if vis_text.contains("pub") {
                        is_public = true;
                    }
                }
            }
        }

        scope.fields.insert(
            field_name.to_string(),
            FieldInfo {
                name: field_name.to_string(),
                is_public,
            },
        );
    }

    fn extract_derives(&self, struct_node: Node, out: &mut HashSet<String>) {
        // Method 1: Check children (inner attributes)
        let mut cursor = struct_node.walk();
        for child in struct_node.children(&mut cursor) {
             self.process_attribute_node(child, out);
        }

        // Method 2: Check previous siblings (outer attributes)
        // Attributes like #[derive] appear before the struct_item as siblings in some TS parsers,
        // or effectively as siblings in the CST if they aren't wrapped.
        let mut prev = struct_node.prev_sibling();
        while let Some(node) = prev {
            if node.kind() == "attribute_item" {
                 self.process_attribute_node(node, out);
                 prev = node.prev_sibling();
            } else if node.kind() == "line_comment" || node.kind() == "block_comment" || node.kind() == "doc_comment" {
                 // Skip comments
                 prev = node.prev_sibling();
            } else {
                 // Met something else (a previous struct, or whitespace?), stop.
                 // Note: tree-sitter usually doesn't yield whitespace nodes unless configured.
                 break;
            }
        }
    }

    fn process_attribute_node(&self, node: Node, out: &mut HashSet<String>) {
        if node.kind() == "attribute_item" {
            let text = node.utf8_text(self.source.as_bytes()).unwrap_or("");
            if text.contains("derive") {
                     let content = text.replace("#[derive(", "").replace(")]", "");
                     for trait_name in content.split(',') {
                         // Remove newlines/spaces
                         let t = trait_name.trim().to_string();
                         if !t.is_empty() {
                            out.insert(t);
                         }
                     }
            }
        }
    }

    fn extract_rust_type_defs(
        &self,
        root: Node,
        out: &mut HashMap<String, Scope>,
        query_str: &str,
        is_enum: bool,
    ) {
        let Ok(query) = Query::new(self.lang.grammar(), query_str) else {
            return;
        };
        let mut cursor = QueryCursor::new();

        for m in cursor.matches(&query, root, self.source.as_bytes()) {
            if let Some(cap) = m.captures.first() {
                if let Ok(name) = cap.node.utf8_text(self.source.as_bytes()) {
                    let row = cap.node.start_position().row + 1;
                    let scope = if is_enum {
                        Scope::new_enum(name, row)
                    } else {
                        Scope::new(name, row)
                    };
                    out.insert(name.to_string(), scope);
                }
            }
        }
    }

    fn extract_rust_impls(&self, root: Node, out: &mut HashMap<String, Scope>) {
        let q_str = "(impl_item type: (type_identifier) @name body: (declaration_list) @body)";
        let Ok(query) = Query::new(self.lang.grammar(), q_str) else {
            return;
        };
        let mut cursor = QueryCursor::new();

        for m in cursor.matches(&query, root, self.source.as_bytes()) {
            self.process_impl_match(&m, out);
        }
    }

    fn process_impl_match(&self, m: &tree_sitter::QueryMatch, out: &mut HashMap<String, Scope>) {
        let mut name = String::new();
        let mut body_node = None;
        let mut name_row = 1;

        for cap in m.captures {
            if cap.index == 0 {
                name = cap
                    .node
                    .utf8_text(self.source.as_bytes())
                    .unwrap_or("")
                    .to_string();
                name_row = cap.node.start_position().row + 1;
            } else if cap.index == 1 {
                body_node = Some(cap.node);
            }
        }

        if let Some(body) = body_node {
            let scope = out
                .entry(name.clone())
                .or_insert_with(|| Scope::new(&name, name_row));
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
        // CRITICAL: Only include instance methods in LCOM4 calculation.
        // Per Hitz & Montazeri (1995), LCOM4 measures cohesion of methods that
        // "share instance variables". Associated functions and constructors
        // don't access instance state and should be excluded.
        // Returns None if static, Some(is_mut) if instance.
        let is_mutable = Self::get_self_mutability(node, self.source)?;

        let name_node = node.child_by_field_name("name")?;
        let name = name_node
            .utf8_text(self.source.as_bytes())
            .ok()?
            .to_string();

        let mut method = Method {
            name,
            field_access: HashSet::new(),
            internal_calls: HashSet::new(),
            external_calls: HashSet::new(),
            cognitive_complexity: CognitiveAnalyzer::calculate(node, self.source),
            is_mutable,
        };

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            self.walk_body_recursive(body, &mut cursor, &mut method);
        }

        Some(method)
    }

    /// Checks if a function has a self parameter (is an instance method).
    /// Returns: Some(true) if mutable (&mut self), Some(false) if immutable (&self), None if distinct.
    fn get_self_mutability(node: Node, source: &str) -> Option<bool> {
        let params = node.child_by_field_name("parameters")?;
        let mut cursor = params.walk();
        for child in params.children(&mut cursor) {
            if child.kind() == "self_parameter" {
                // Check if it contains "mut"
                let text = child.utf8_text(source.as_bytes()).unwrap_or("");
                return Some(text.contains("mut"));
            }
        }
        None
    }

    fn walk_body_recursive(&self, node: Node, cursor: &mut TreeCursor, method: &mut Method) {
        self.check_node_heuristics(node, method);

        if cursor.goto_first_child() {
            loop {
                self.walk_body_recursive(cursor.node(), cursor, method);
                if !cursor.goto_next_sibling() {
                    break;
                }
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
                        method
                            .internal_calls
                            .insert(call_target.replace("self.", ""));
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
        function
            .utf8_text(self.source.as_bytes())
            .ok()
            .map(String::from)
    }
}
