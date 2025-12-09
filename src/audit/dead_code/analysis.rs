use tree_sitter::Node;

/// Analyzes imports and references to build edge list.
#[must_use]
pub fn extract_references(
    source: &str,
    _file: &std::path::Path,
    tree: &tree_sitter::Tree,
) -> Vec<(String, String)> {
    let mut refs = Vec::new();
    let source_bytes = source.as_bytes();

    extract_refs_from_node(tree.root_node(), source_bytes, &mut refs, None);

    refs
}

fn extract_refs_from_node(
    node: Node,
    source: &[u8],
    refs: &mut Vec<(String, String)>,
    current_fn: Option<&str>,
) {
    let kind = node.kind();
    let new_fn = helper_extract_fn_name(node, source).or(current_fn.map(String::from));
    let current_fn_str = new_fn.as_deref();

    if let Some(caller) = current_fn_str {
        process_node_kind(node, kind, source, caller, refs);
    }

    for child in node.children(&mut node.walk()) {
        extract_refs_from_node(child, source, refs, current_fn_str);
    }
}

fn process_node_kind(
    node: Node,
    kind: &str,
    source: &[u8],
    caller: &str,
    refs: &mut Vec<(String, String)>,
) {
    match kind {
        "call_expression" => process_call(node, source, caller, refs),
        "identifier" | "field_identifier" => process_identifier(node, source, caller, refs),
        _ => {}
    }
}

fn process_call(
    node: Node,
    source: &[u8],
    caller: &str,
    refs: &mut Vec<(String, String)>,
) {
    if let Some(callee) = extract_call_target(node, source) {
        refs.push((caller.to_string(), callee));
    }
}

fn process_identifier(
    node: Node,
    source: &[u8],
    caller: &str,
    refs: &mut Vec<(String, String)>,
) {
    let Ok(name) = node.utf8_text(source) else {
        return;
    };

    if is_common_identifier(name) {
        return;
    }

    refs.push((caller.to_string(), name.to_string()));
}


fn helper_extract_fn_name(node: Node, source: &[u8]) -> Option<String> {
    let kind = node.kind();
    if kind == "function_item" || kind == "function_definition" {
        node.child_by_field_name("name")
            .and_then(|n| n.utf8_text(source).ok())
            .map(String::from)
    } else {
        None
    }
}

fn extract_call_target(node: Node, source: &[u8]) -> Option<String> {
    let target = node.child_by_field_name("function")?;

    match target.kind() {
        "identifier" | "scoped_identifier" => target.utf8_text(source).ok().map(String::from),
        "field_expression" => target
            .child_by_field_name("field")
            .and_then(|n| n.utf8_text(source).ok())
            .map(String::from),
        _ => None,
    }
}

fn is_common_identifier(name: &str) -> bool {
    matches!(
        name,
        "self" | "Self" | "super" | "crate" | "true" | "false" | "None" | "Some" | "Ok" | "Err"
    )
}
