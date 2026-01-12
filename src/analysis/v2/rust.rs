// src/analysis/v2/rust.rs
use super::scope::{FieldInfo, Scope};
use super::rust_impl::RustImplExtractor;
use tree_sitter::{Node, Query, QueryCursor};

pub struct RustExtractor;

impl RustExtractor {
    pub fn extract_scopes(source: &str, root: Node, out: &mut std::collections::HashMap<String, Scope>) {
        Self::extract_structs_and_fields(source, root, out);
        Self::extract_type_defs(source, root, out, "(enum_item name: (type_identifier) @name)", true);
        RustImplExtractor::extract(source, root, out);
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
        let mut cursor = field_node.walk();
        for child in field_node.children(&mut cursor) {
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
}