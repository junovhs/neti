use crate::harvester::{normalize_string, Collector};
use tree_sitter::Node;

#[path = "harvester_signatures.rs"]
mod signatures;

pub(crate) fn walk(
    node: Node<'_>,
    source: &str,
    collector: &mut Collector,
    ext: &str,
    exported: bool,
) {
    let kind = node.kind();
    let text = text_for(node, source);
    let is_exported = exported || node_is_exported(node, source, ext);

    match ext {
        "go" => collect_go(node, kind, &text, source, collector, is_exported),
        "rs" => collect_rust(node, kind, &text, source, collector, is_exported),
        "py" => collect_python(node, kind, &text, source, collector, is_exported),
        "ts" | "tsx" | "js" | "jsx" | "mjs" => {
            collect_typescript(node, kind, &text, source, collector, is_exported)
        }
        "c" | "cc" | "cpp" | "cxx" | "h" | "hh" | "hpp" | "hxx" => {
            collect_cpp(node, kind, &text, source, collector, is_exported)
        }
        _ => {}
    }

    if kind.contains("comment") {
        collector.comments.push(text.trim().to_string());
    }
    let _ = (kind.contains("string") || kind == "string_literal" || kind == "raw_string_literal")
        && normalize_string(&text)
            .map(|value| collector.strings.insert(value))
            .is_some();

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(child, source, collector, ext, is_exported);
    }
}

fn collect_go(
    node: Node<'_>,
    kind: &str,
    _text: &str,
    source: &str,
    collector: &mut Collector,
    is_exported: bool,
) {
    match kind {
        "import_spec" => maybe_insert(&mut collector.imports, quoted_child_value(node, source)),
        "field_declaration" => {
            maybe_insert(&mut collector.annotations, child_text(node, source, "tag"))
        }
        "function_declaration" | "method_declaration" if is_exported => {
            signatures::collect_function_signature(node, source, collector, "parameters", "result");
        }
        _ => {}
    }
}

fn collect_rust(
    node: Node<'_>,
    kind: &str,
    text: &str,
    source: &str,
    collector: &mut Collector,
    is_exported: bool,
) {
    match kind {
        "use_declaration" => {
            let trimmed = text
                .trim()
                .trim_end_matches(';')
                .trim_start_matches("use ")
                .trim();
            maybe_insert(
                &mut collector.imports,
                (!trimmed.is_empty()).then(|| trimmed.to_string()),
            );
        }
        "attribute_item" => collect_rust_attribute(text, collector),
        "function_item" if is_exported => {
            signatures::collect_function_signature(
                node,
                source,
                collector,
                "parameters",
                "return_type",
            );
        }
        _ => {}
    }
}

fn collect_rust_attribute(text: &str, collector: &mut Collector) {
    let trimmed = text.trim().trim_start_matches("#[").trim_end_matches(']');
    if let Some(derive_args) = trimmed.strip_prefix("derive(") {
        let names = derive_args.trim_end_matches(')');
        for name in names.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            collector.annotations.insert(name.to_string());
        }
    } else if let Some((name, _)) = trimmed.split_once('(') {
        collector.annotations.insert(name.trim().to_string());
    } else if !trimmed.is_empty() {
        collector.annotations.insert(trimmed.to_string());
    }
}

fn collect_python(
    node: Node<'_>,
    kind: &str,
    text: &str,
    source: &str,
    collector: &mut Collector,
    is_exported: bool,
) {
    match kind {
        "import_statement" => {
            for item in text
                .trim()
                .trim_start_matches("import ")
                .trim()
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                collector.imports.insert(item.to_string());
            }
        }
        "import_from_statement" => maybe_insert(
            &mut collector.imports,
            text.trim()
                .strip_prefix("from ")
                .and_then(|rest| rest.split_whitespace().next())
                .map(str::to_string),
        ),
        "decorator" => collect_decorator(text, collector),
        "function_definition" if is_exported => {
            signatures::collect_python_signature(node, source, collector)
        }
        _ => {}
    }
}

fn collect_typescript(
    node: Node<'_>,
    kind: &str,
    text: &str,
    source: &str,
    collector: &mut Collector,
    is_exported: bool,
) {
    match kind {
        "import_statement" => maybe_insert(&mut collector.imports, extract_import_path(text)),
        "decorator" => collect_decorator(text, collector),
        "function_declaration" | "method_definition" if is_exported => {
            signatures::collect_function_signature(
                node,
                source,
                collector,
                "parameters",
                "return_type",
            );
        }
        _ => {}
    }
}

fn collect_cpp(
    node: Node<'_>,
    kind: &str,
    text: &str,
    source: &str,
    collector: &mut Collector,
    is_exported: bool,
) {
    match kind {
        "preproc_include" => {
            let normalized = text
                .trim()
                .trim_start_matches("#include")
                .trim()
                .trim_matches('"')
                .trim_matches('<')
                .trim_matches('>');
            maybe_insert(
                &mut collector.imports,
                (!normalized.is_empty()).then(|| normalized.to_string()),
            );
        }
        "function_definition" if is_exported => {
            signatures::collect_cpp_signature(node, source, collector)
        }
        _ => {}
    }
}

fn collect_decorator(text: &str, collector: &mut Collector) {
    let name = text.trim().trim_start_matches('@');
    let simple = name.split('(').next().unwrap_or(name).trim();
    maybe_insert(
        &mut collector.annotations,
        (!simple.is_empty()).then(|| simple.to_string()),
    );
}

fn child_text(node: Node<'_>, source: &str, field: &str) -> Option<String> {
    node.child_by_field_name(field)
        .map(|child| text_for(child, source).trim().to_string())
}

fn quoted_child_value(node: Node<'_>, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    let found = node
        .children(&mut cursor)
        .find_map(|child| normalize_string(&text_for(child, source)));
    found
}

fn text_for(node: Node<'_>, source: &str) -> String {
    source.get(node.byte_range()).unwrap_or("").to_string()
}

fn node_is_exported(node: Node<'_>, source: &str, ext: &str) -> bool {
    match ext {
        "go" => node
            .child_by_field_name("name")
            .map(|name| starts_uppercase(&text_for(name, source)))
            .unwrap_or(false),
        "rs" => text_for(node, source).trim_start().starts_with("pub "),
        "py" => node
            .child_by_field_name("name")
            .map(|name| !text_for(name, source).trim_start().starts_with('_'))
            .unwrap_or(false),
        "ts" | "tsx" | "js" | "jsx" | "mjs" => {
            let text = text_for(node, source);
            text.trim_start().starts_with("export ")
                || node
                    .parent()
                    .map(|parent| parent.kind() == "export_statement")
                    .unwrap_or(false)
        }
        "c" | "cc" | "cpp" | "cxx" | "h" | "hh" | "hpp" | "hxx" => {
            !text_for(node, source).contains("static ")
        }
        _ => false,
    }
}

fn starts_uppercase(s: &str) -> bool {
    s.chars().next().is_some_and(char::is_uppercase)
}

fn extract_import_path(raw: &str) -> Option<String> {
    let start = raw.find('"').or_else(|| raw.find('\''))?;
    let quote = raw.as_bytes().get(start).copied()? as char;
    let rest = raw.get(start + 1..)?;
    let end = rest.find(quote)?;
    Some(rest[..end].to_string())
}

fn maybe_insert(output: &mut std::collections::BTreeSet<String>, value: Option<String>) {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        output.insert(value);
    }
}
