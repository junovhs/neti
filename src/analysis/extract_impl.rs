//! Rust impl/method extraction logic.
//! Separated from extract.rs to satisfy Law of Atomicity (token limit).

use super::cognitive::CognitiveAnalyzer;
use super::scope::{Method, Scope};
use std::collections::HashMap;
use tree_sitter::{Node, Query, QueryCursor, TreeCursor};

#[allow(clippy::implicit_hasher)]
pub fn extract(source: &str, root: Node, out: &mut HashMap<String, Scope>) {
    let q_str = "(impl_item type: (type_identifier) @name body: (declaration_list) @body)";
    let Ok(query) = Query::new(tree_sitter_rust::language(), q_str) else {
        return;
    };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        process_impl_match(source, &m, out);
    }
}

fn process_impl_match(source: &str, m: &tree_sitter::QueryMatch, out: &mut HashMap<String, Scope>) {
    let mut name = String::new();
    let mut body_node = None;
    let mut name_row = 1;

    for cap in m.captures {
        if cap.index == 0 {
            name = cap
                .node
                .utf8_text(source.as_bytes())
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
        process_impl_body(source, body, scope);
    }
}

fn process_impl_body(source: &str, body: Node, scope: &mut Scope) {
    let mut cursor = body.walk();
    for child in body.children(&mut cursor) {
        if child.kind() == "function_item" {
            if let Some(method) = extract_method(source, child) {
                scope.add_method(method);
            }
        }
    }
}

fn extract_method(source: &str, node: Node) -> Option<Method> {
    let is_mutable = get_self_mutability(node, source)?;
    let name_node = node.child_by_field_name("name")?;
    let name = name_node.utf8_text(source.as_bytes()).ok()?.to_string();
    let mut method = Method::new(
        &name,
        CognitiveAnalyzer::calculate(node, source),
        is_mutable,
    );
    if let Some(body) = node.child_by_field_name("body") {
        let mut cursor = body.walk();
        walk_body_recursive(source, body, &mut cursor, &mut method);
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
    check_node_heuristics(source, node, method);
    if cursor.goto_first_child() {
        loop {
            walk_body_recursive(source, cursor.node(), cursor, method);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}

fn check_node_heuristics(source: &str, node: Node, method: &mut Method) {
    match node.kind() {
        "field_expression" => {
            if let Some(field) = extract_field(source, node) {
                method.field_access.insert(field);
            }
        }
        "call_expression" => {
            if let Some(call_target) = extract_call(source, node) {
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

fn extract_field(source: &str, node: Node) -> Option<String> {
    let val = node.child_by_field_name("value")?;
    let field = node.child_by_field_name("field")?;
    if val.utf8_text(source.as_bytes()).ok()? == "self" {
        return Some(field.utf8_text(source.as_bytes()).ok()?.to_string());
    }
    None
}

fn extract_call(source: &str, node: Node) -> Option<String> {
    let function = node.child_by_field_name("function")?;
    function.utf8_text(source.as_bytes()).ok().map(String::from)
}
