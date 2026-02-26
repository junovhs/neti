// src/analysis/patterns/logic_proof.rs
//! Fixed-size array proof helpers for L03.
//!
//! Proves that constant indexing into fixed-size arrays is safe,
//! so L03 can skip violations like `seed[0]` on `[0u8; 32]`.

use tree_sitter::Node;

use super::logic_helpers::decl_matches_variable;

#[cfg(test)]
#[path = "logic_proof_test.rs"]
mod tests;

#[path = "logic_proof_helpers.rs"]
mod helpers;

/// Returns `true` if the index expression is provably safe because
/// the receiver is a fixed-size array and the index is within bounds.
pub fn is_fixed_size_array_access(source: &str, idx_node: Node, root: Node) -> bool {
    let text = idx_node.utf8_text(source.as_bytes()).unwrap_or("");

    let Some(index_val) = extract_constant_index(text) else {
        return false;
    };

    let receiver = extract_receiver(text);

    if let Some(size) = find_local_array_size(source, idx_node, receiver) {
        return index_val < size;
    }

    if let Some(field_name) = receiver.strip_prefix("self.") {
        if !field_name.contains('.') {
            if let Some(size) =
                helpers::find_struct_field_array_size(source, idx_node, root, field_name)
            {
                return index_val < size;
            }
        }
    }

    if let Some(size) = helpers::find_param_array_size(source, idx_node, receiver) {
        return index_val < size;
    }

    false
}

/// Extract the receiver portion of an index expression (everything before `[`).
pub fn extract_receiver(text: &str) -> &str {
    text.rfind('[').map_or(text, |pos| text[..pos].trim())
}

fn extract_constant_index(text: &str) -> Option<usize> {
    let bracket_start = text.rfind('[')?;
    let bracket_end = text.rfind(']')?;
    if bracket_end <= bracket_start {
        return None;
    }
    let inner = text[bracket_start + 1..bracket_end].trim();
    inner.parse::<usize>().ok()
}

/// Search a single scope node for a let declaration that matches `receiver`
/// and comes before `node`.
fn scope_let_array_size(source: &str, node: Node, scope: Node, receiver: &str) -> Option<usize> {
    let mut child_cursor = scope.walk();
    for child in scope.children(&mut child_cursor) {
        if child.kind() != "let_declaration" {
            continue;
        }
        if child.start_byte() >= node.start_byte() {
            continue;
        }
        let decl_text = child.utf8_text(source.as_bytes()).unwrap_or("");
        if !decl_matches_variable(decl_text, receiver) {
            continue;
        }
        if let Some(size) = extract_array_size_from_decl(decl_text) {
            return Some(size);
        }
    }
    None
}

fn find_local_array_size(source: &str, node: Node, receiver: &str) -> Option<usize> {
    if receiver.contains('.') {
        return None;
    }

    let mut cur = node;
    for _ in 0..30 {
        let Some(p) = cur.parent() else { break };

        if matches!(p.kind(), "block" | "function_item" | "source_file") {
            if let Some(size) = scope_let_array_size(source, node, p, receiver) {
                return Some(size);
            }
            if matches!(p.kind(), "function_item" | "source_file") {
                break;
            }
        }
        cur = p;
    }
    None
}

fn extract_array_size_from_decl(decl_text: &str) -> Option<usize> {
    extract_repeat_array_size(decl_text)
        .or_else(|| extract_type_array_size(decl_text))
        .or_else(|| extract_literal_array_size(decl_text))
}

fn extract_repeat_array_size(text: &str) -> Option<usize> {
    let eq_pos = text.find('=')?;
    let after_eq = text[eq_pos + 1..].trim();
    if !after_eq.starts_with('[') {
        return None;
    }
    let bracket_end = after_eq.find(']')?;
    let inner = &after_eq[1..bracket_end];
    let semi_pos = inner.rfind(';')?;
    let size_str = inner[semi_pos + 1..].trim();
    parse_size_literal(size_str)
}

fn extract_type_array_size(text: &str) -> Option<usize> {
    let colon_pos = text.find(':')?;
    let after_colon = &text[colon_pos + 1..];
    let eq_pos = after_colon.find('=').unwrap_or(after_colon.len());
    let type_region = &after_colon[..eq_pos];

    let bracket_start = type_region.find('[')?;
    let bracket_end = type_region.find(']')?;
    if bracket_end <= bracket_start {
        return None;
    }
    let inner = &type_region[bracket_start + 1..bracket_end];
    let semi_pos = inner.rfind(';')?;
    let size_str = inner[semi_pos + 1..].trim();
    parse_size_literal(size_str)
}

fn extract_literal_array_size(text: &str) -> Option<usize> {
    let eq_pos = text.find('=')?;
    let after_eq = text[eq_pos + 1..].trim();
    if !after_eq.starts_with('[') {
        return None;
    }
    let bracket_end = after_eq.find(']')?;
    let inner = &after_eq[1..bracket_end];
    if inner.contains(';') {
        return None;
    }
    let trimmed = inner.trim();
    if trimmed.is_empty() {
        return Some(0);
    }
    Some(trimmed.split(',').count())
}

fn parse_size_literal(s: &str) -> Option<usize> {
    let cleaned = s
        .trim()
        .trim_end_matches("usize")
        .trim_end_matches("u32")
        .trim_end_matches("u64")
        .trim_end_matches("i32")
        .trim_end_matches("i64")
        .trim();
    cleaned.parse::<usize>().ok()
}
