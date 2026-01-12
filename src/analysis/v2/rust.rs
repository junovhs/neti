// src/analysis/v2/rust.rs
use super::cognitive::CognitiveAnalyzer;
use super::scope::{FieldInfo, Method, Scope};
use tree_sitter::{Node, Query, QueryCursor, TreeCursor};

pub struct RustExtractor;

impl RustExtractor {
    pub fn extract_scopes(source: &str, root: Node, out: &mut std::collections::HashMap<String, Scope>) {
        Self::extract_structs_and_fields(source, root, out);
        Self::extract_type_defs(source, root, out, "(enum_item name: (type_identifier) @name)", true);
        Self::extract_impls(source, root, out);
    }

    fn extract_structs_and_fields(source: &str, root: Node, out: &mut std::collections::HashMap<String, Scope>) {
        let query_str = "(field_declaration) @field";
        let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else { return };
        let mut cursor = QueryCursor::new();

        for m in cursor.matches(&query, root, source.as_bytes()) {
            for cap in m.captures {
                Self::process_field_node(source, cap.node, out);
            }
        }
    }

    fn process_field_node(source: &str, field_node: Node, out: &mut std::collections::HashMap<String, Scope>) {
        let Some(field_list) = field_node.parent() else { return };
        let Some(struct_node) = field_list.parent() else { return };
        if struct_node.kind() != "struct_item" { return; }

        let Some(name_node) = struct_node.child_by_field_name("name") else { return };
        let Ok(struct_name) = name_node.utf8_text(source.as_bytes()) else { return };
        let struct_row = struct_node.start_position().row + 1;

        let scope = out
            .entry(struct_name.to_string())
            .or_insert_with(|| Scope::new(struct_name, struct_row));

        if !scope.has_derives() {
            Self::extract_derives(source, struct_node, scope);
        }

        let Some(field_name_node) = field_node.child_by_field_name("name") else { return };
        let Ok(field_name) = field_name_node.utf8_text(source.as_bytes()) else { return };
        
        let mut is_public = false;
        for child in field_node.children(&mut field_node.walk()) {
            if child.kind() == "visibility_modifier" {
                if let Ok(vis_text) = child.utf8_text(source.as_bytes()) {
                    if vis_text.contains("pub") { is_public = true; }
                }
            }
        }

        scope.add_field(field_name.to_string(), FieldInfo::new(field_name, is_public));
    }

    fn extract_derives(source: &str, struct_node: Node, scope: &mut Scope) {
        let mut cursor = struct_node.walk();
        for child in struct_node.children(&mut cursor) {
             Self::process_attribute_node(source, child, scope);
        }

        let mut prev = struct_node.prev_sibling();
        while let Some(node) = prev {
            if node.kind() == "attribute_item" {
                 Self::process_attribute_node(source, node, scope);
                 prev = node.prev_sibling();
            } else if node.kind() == "line_comment" || node.kind() == "block_comment" || node.kind() == "doc_comment" {
                 prev = node.prev_sibling();
            } else { break; }
        }
    }

    fn process_attribute_node(source: &str, node: Node, scope: &mut Scope) {
        if node.kind() == "attribute_item" {
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");
            if text.contains("derive") {
                 let content = text.replace("#[derive(", "").replace(")]", "");
                 for trait_name in content.split(',') {
                     let t = trait_name.trim().to_string();
                     if !t.is_empty() { scope.add_derive(t); }
                 }
            }
        }
    }

    fn extract_type_defs(source: &str, root: Node, out: &mut std::collections::HashMap<String, Scope>, query_str: &str, is_enum: bool) {
        let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else { return };
        let mut cursor = QueryCursor::new();
        for m in cursor.matches(&query, root, source.as_bytes()) {
            if let Some(cap) = m.captures.first() {
                if let Ok(name) = cap.node.utf8_text(source.as_bytes()) {
                    let row = cap.node.start_position().row + 1;
                    out.insert(name.to_string(), if is_enum { Scope::new_enum(name, row) } else { Scope::new(name, row) });
                }
            }
        }
    }

    fn extract_impls(source: &str, root: Node, out: &mut std::collections::HashMap<String, Scope>) {
        let q_str = "(impl_item type: (type_identifier) @name body: (declaration_list) @body)";
        let Ok(query) = Query::new(tree_sitter_rust::language(), q_str) else { return };
        let mut cursor = QueryCursor::new();
        for m in cursor.matches(&query, root, source.as_bytes()) {
            Self::process_impl_match(source, &m, out);
        }
    }

    fn process_impl_match(source: &str, m: &tree_sitter::QueryMatch, out: &mut std::collections::HashMap<String, Scope>) {
        let mut name = String::new();
        let mut body_node = None;
        let mut name_row = 1;
        for cap in m.captures {
            if cap.index == 0 {
                name = cap.node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                name_row = cap.node.start_position().row + 1;
            } else if cap.index == 1 { body_node = Some(cap.node); }
        }
        if let Some(body) = body_node {
            let scope = out.entry(name.clone()).or_insert_with(|| Scope::new(&name, name_row));
            Self::process_impl_body(source, body, scope);
        }
    }

    fn process_impl_body(source: &str, body: Node, scope: &mut Scope) {
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            if child.kind() == "function_item" {
                if let Some(method) = Self::extract_method(source, child) {
                    // P01: Avoid cloning in loop if possible, but here we construct a HashMap entry.
                    // The clone of method.name is necessary for the key.
                    // But wait, `method` has `name` field.
                    // We can clone just once.
                    let name_key = method.name.clone();
                    scope.add_method(name_key, method);
                }
            }
        }
    }

    fn extract_method(source: &str, node: Node) -> Option<Method> {
        let is_mutable = Self::get_self_mutability(node, source)?;
        let name_node = node.child_by_field_name("name")?;
        let name = name_node.utf8_text(source.as_bytes()).ok()?.to_string();
        let mut method = Method::new(&name, CognitiveAnalyzer::calculate(node, source), is_mutable);
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            Self::walk_body_recursive(source, body, &mut cursor, &mut method);
        }
        Some(method)
    }

    fn get_self_mutability(node: Node, source: &str) -> Option<bool> {
        let params = node.child_by_field_name("parameters")?;
        let mut cursor = params.walk();
        for child in params.children(&mut cursor) {
            if child.kind() == "self_parameter" {
                let text = child.utf8_text(source.as_bytes()).unwrap_or("");
                return Some(text.contains("mut"));
            }
        }
        None
    }

    fn walk_body_recursive(source: &str, node: Node, cursor: &mut TreeCursor, method: &mut Method) {
        Self::check_node_heuristics(source, node, method);
        if cursor.goto_first_child() {
            loop {
                Self::walk_body_recursive(source, cursor.node(), cursor, method);
                if !cursor.goto_next_sibling() { break; }
            }
            cursor.goto_parent();
        }
    }

    fn check_node_heuristics(source: &str, node: Node, method: &mut Method) {
        match node.kind() {
            "field_expression" => if let Some(field) = Self::extract_field(source, node) { method.field_access.insert(field); },
            "call_expression" => if let Some(call_target) = Self::extract_call(source, node) {
                if call_target.starts_with("self.") { method.internal_calls.insert(call_target.replace("self.", "")); }
                else { method.external_calls.insert(call_target); }
            },
            _ => {}
        }
    }

    fn extract_field(source: &str, node: Node) -> Option<String> {
        let val = node.child_by_field_name("value")?;
        let field = node.child_by_field_name("field")?;
        if val.utf8_text(source.as_bytes()).ok()? == "self" { return Some(field.utf8_text(source.as_bytes()).ok()?.to_string()); }
        None
    }

    fn extract_call(source: &str, node: Node) -> Option<String> {
        let function = node.child_by_field_name("function")?;
        function.utf8_text(source.as_bytes()).ok().map(String::from)
    }
}