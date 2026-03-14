use crate::harvester::{normalize_type_text, Collector};
use std::collections::BTreeSet;
use tree_sitter::{Node, TreeCursor};

pub(crate) fn collect_function_signature(
    node: Node<'_>,
    source: &str,
    collector: &mut Collector,
    params_field: &str,
    return_field: &str,
) {
    if let Some(params) = node.child_by_field_name(params_field) {
        collect_type_texts(params, source, &mut collector.param_types);
    }
    if let Some(ret) = node.child_by_field_name(return_field) {
        collect_type_texts(ret, source, &mut collector.return_types);
    }
}

pub(crate) fn collect_python_signature(node: Node<'_>, source: &str, collector: &mut Collector) {
    if let Some(params) = node.child_by_field_name("parameters") {
        let mut cursor = params.walk();
        let param_types: Vec<String> = params
            .children(&mut cursor)
            .filter(|child| child.kind().contains("parameter") || child.kind() == "typed_parameter")
            .filter_map(|child| {
                let text = text_for(child, source);
                let (_, ty) = text.split_once(':')?;
                let cleaned = ty.trim().trim_end_matches(',').trim();
                (!cleaned.is_empty()).then(|| cleaned.to_string())
            })
            .collect();
        collector.param_types.extend(param_types);
    }
    if let Some(ret) = node.child_by_field_name("return_type") {
        let ret = text_for(ret, source);
        let ret = ret.trim().trim_start_matches("->").trim();
        if !ret.is_empty() {
            collector.return_types.insert(ret.to_string());
        }
    }
}

pub(crate) fn collect_cpp_signature(node: Node<'_>, source: &str, collector: &mut Collector) {
    if let Some(declarator) = node.child_by_field_name("declarator") {
        collect_type_texts(declarator, source, &mut collector.param_types);
    }
    if let Some(ty) = node.child_by_field_name("type") {
        let text = text_for(ty, source);
        let text = text.trim();
        if !text.is_empty() {
            collector.return_types.insert(text.to_string());
        }
    }
}

fn collect_type_texts(node: Node<'_>, source: &str, output: &mut BTreeSet<String>) {
    if is_type_node(node.kind()) {
        let text = normalize_type_text(&text_for(node, source));
        if !text.is_empty() {
            output.insert(text);
        }
    }
    let mut cursor: TreeCursor<'_> = node.walk();
    for child in node.children(&mut cursor) {
        collect_type_texts(child, source, output);
    }
}

fn is_type_node(kind: &str) -> bool {
    kind == "type_identifier"
        || kind.ends_with("_type")
        || kind == "qualified_type"
        || kind == "generic_type"
        || kind == "type_annotation"
        || kind == "parameter_declaration"
        || kind == "typed_parameter"
        || kind == "variadic_parameter"
}

fn text_for(node: Node<'_>, source: &str) -> String {
    source.get(node.byte_range()).unwrap_or("").to_string()
}
