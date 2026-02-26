// src/analysis/patterns/logic_proof_helpers.rs
//! Helper routines for extracting and verifying array sizes in scope boundaries.

use tree_sitter::Node;

pub(super) fn find_struct_field_array_size(
    source: &str,
    node: Node,
    root: Node,
    field_name: &str,
) -> Option<usize> {
    let type_name = find_enclosing_impl_type(source, node)?;

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() != "struct_item" {
            continue;
        }
        let struct_text = child.utf8_text(source.as_bytes()).unwrap_or("");
        if !struct_text.contains(&format!("struct {type_name}"))
            && !struct_text.contains(&format!("struct {type_name}<"))
        {
            continue;
        }
        for line in struct_text.lines() {
            if let Some(size) = extract_field_array_size(line, field_name) {
                return Some(size);
            }
        }
    }
    None
}

fn find_enclosing_impl_type<'a>(source: &'a str, node: Node) -> Option<&'a str> {
    let mut cur = node;
    for _ in 0..30 {
        let Some(p) = cur.parent() else { break };
        if p.kind() == "impl_item" {
            let impl_text = p.utf8_text(source.as_bytes()).unwrap_or("");
            return extract_impl_type_name(impl_text);
        }
        if p.kind() == "source_file" {
            break;
        }
        cur = p;
    }
    None
}

pub(super) fn extract_impl_type_name(impl_text: &str) -> Option<&str> {
    let first_line = impl_text.lines().next()?;
    let after_impl = first_line.strip_prefix("impl")?.trim();

    let after_generics = if after_impl.starts_with('<') {
        skip_angle_brackets(after_impl)
    } else {
        after_impl
    };

    let type_part = if let Some(pos) = after_generics.find(" for ") {
        after_generics[pos + 5..].trim()
    } else {
        after_generics
    };

    let name = type_part
        .split(|c: char| c == '<' || c == '{' || c.is_whitespace())
        .next()?;

    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn skip_angle_brackets(text: &str) -> &str {
    let mut depth = 0;
    for (i, c) in text.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    return text[i + 1..].trim();
                }
            }
            _ => {}
        }
    }
    text
}

pub(super) fn extract_field_array_size(line: &str, field_name: &str) -> Option<usize> {
    let trimmed = line.trim();
    let after_pub = trimmed.strip_prefix("pub ").unwrap_or(trimmed).trim();
    if !after_pub.starts_with(field_name) {
        return None;
    }
    let after_name = &after_pub[field_name.len()..];
    if !after_name.starts_with(':') {
        return None;
    }
    let type_str = after_name[1..].trim();
    if !type_str.starts_with('[') {
        return None;
    }
    let bracket_end = type_str.find(']')?;
    let inner = &type_str[1..bracket_end];
    let semi_pos = inner.rfind(';')?;
    let size_str = inner[semi_pos + 1..].trim().trim_end_matches(',');
    size_str.parse::<usize>().ok()
}

pub(super) fn find_param_array_size(source: &str, node: Node, receiver: &str) -> Option<usize> {
    if receiver.contains('.') {
        return None;
    }

    let mut cur = node;
    for _ in 0..20 {
        let Some(p) = cur.parent() else { break };

        if matches!(p.kind(), "function_item" | "closure_expression") {
            return extract_param_array_size(source, p, receiver);
        }
        cur = p;
    }
    None
}

fn extract_param_array_size(source: &str, fn_node: Node, receiver: &str) -> Option<usize> {
    let fn_text = fn_node.utf8_text(source.as_bytes()).unwrap_or("");
    let pattern = format!("{receiver}:");
    let pos = fn_text.find(&pattern)?;
    let after = fn_text[pos + pattern.len()..].trim();

    if !after.starts_with('[') {
        return None;
    }
    let bracket_end = after.find(']')?;
    let inner = &after[1..bracket_end];
    let semi_pos = inner.rfind(';')?;
    let size_str = inner[semi_pos + 1..].trim();
    size_str.parse::<usize>().ok()
}
