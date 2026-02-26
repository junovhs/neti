// src/analysis/extract.rs
//! Rust scope extraction logic (Structs/Enums/Fields).
//! Method extraction moved to `extract_impl.rs` for Atomicity.

use super::scope::{FieldInfo, Scope};
use tree_sitter::{Node, Query, QueryCursor};

pub struct RustExtractor;

impl RustExtractor {
    pub fn extract_scopes(
        source: &str,
        root: Node,
        out: &mut std::collections::HashMap<String, Scope>,
    ) {
        Self::extract_structs_and_fields(source, root, out);
        Self::extract_type_defs(
            source,
            root,
            out,
            "(enum_item name: (type_identifier) @name)",
            true,
        );
        // Delegate impl extraction to separate module
        super::extract_impl::extract(source, root, out);
    }

    fn extract_structs_and_fields(
        source: &str,
        root: Node,
        out: &mut std::collections::HashMap<String, Scope>,
    ) {
        let query_str = "(field_declaration) @field";
        let Ok(query) = Query::new(&tree_sitter_rust::LANGUAGE.into(), query_str) else {
            return;
        };
        let mut cursor = QueryCursor::new();
        let matches: Vec<_> = cursor.matches(&query, root, source.as_bytes()).collect();

        for m in &matches {
            for cap in m.captures {
                // neti:allow(P04)
                Self::process_field_node(source, cap.node, out);
            }
        }
    }

    fn process_field_node(
        source: &str,
        field_node: Node,
        out: &mut std::collections::HashMap<String, Scope>,
    ) {
        let Some(field_list) = field_node.parent() else {
            return;
        };
        let Some(struct_node) = field_list.parent() else {
            return;
        };
        if struct_node.kind() != "struct_item" {
            return;
        }

        let Some(name_node) = struct_node.child_by_field_name("name") else {
            return;
        };
        let Ok(struct_name) = name_node.utf8_text(source.as_bytes()) else {
            return;
        };
        let struct_row = struct_node.start_position().row + 1;

        let scope = out
            .entry(struct_name.to_string())
            .or_insert_with(|| Scope::new(struct_name, struct_row));

        if !scope.has_derives() {
            Self::extract_derives(source, struct_node, scope);
        }

        let Some(field_name_node) = field_node.child_by_field_name("name") else {
            return;
        };
        let Ok(field_name) = field_name_node.utf8_text(source.as_bytes()) else {
            return;
        };

        let is_public = check_field_visibility(source, field_node);

        scope.add_field(
            field_name.to_string(),
            FieldInfo::new(field_name, is_public),
        );
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
            } else if node.kind() == "line_comment"
                || node.kind() == "block_comment"
                || node.kind() == "doc_comment"
            {
                prev = node.prev_sibling();
            } else {
                break;
            }
        }
    }

    fn process_attribute_node(source: &str, node: Node, scope: &mut Scope) {
        if node.kind() == "attribute_item" {
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");
            if text.contains("derive") {
                let content = text.replace("#[derive(", "").replace(")]", "");
                for trait_name in content.split(',') {
                    let t = trait_name.trim().to_string();
                    if !t.is_empty() {
                        scope.add_derive(t);
                    }
                }
            }
        }
    }

    fn extract_type_defs(
        source: &str,
        root: Node,
        out: &mut std::collections::HashMap<String, Scope>,
        query_str: &str,
        is_enum: bool,
    ) {
        let Ok(query) = Query::new(&tree_sitter_rust::LANGUAGE.into(), query_str) else {
            return;
        };
        let mut cursor = QueryCursor::new();
        for m in cursor.matches(&query, root, source.as_bytes()) {
            extract_single_type_def(source, &m, out, is_enum);
        }
    }
}

fn extract_single_type_def(
    source: &str,
    m: &tree_sitter::QueryMatch,
    out: &mut std::collections::HashMap<String, Scope>,
    is_enum: bool,
) {
    let Some(cap) = m.captures.first() else {
        return;
    };
    let Ok(name) = cap.node.utf8_text(source.as_bytes()) else {
        return;
    };
    let row = cap.node.start_position().row + 1;
    let name_str = name.to_string();
    out.insert(
        name_str,
        if is_enum {
            Scope::new_enum(name, row)
        } else {
            Scope::new(name, row)
        },
    );
}

fn check_field_visibility(source: &str, field_node: Node) -> bool {
    let mut cursor = field_node.walk();
    for child in field_node.children(&mut cursor) {
        if child.kind() == "visibility_modifier" {
            if let Ok(vis_text) = child.utf8_text(source.as_bytes()) {
                if vis_text.contains("pub") {
                    return true;
                }
            }
        }
    }
    false
}
