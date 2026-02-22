//! Fixed-size array proof helpers for L03.
//!
//! Proves that constant indexing into fixed-size arrays is safe,
//! so L03 can skip violations like `seed[0]` on `[0u8; 32]`.

use tree_sitter::Node;

use super::logic_helpers::decl_matches_variable;

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
            if let Some(size) = find_struct_field_array_size(source, idx_node, root, field_name) {
                return index_val < size;
            }
        }
    }

    if let Some(size) = find_param_array_size(source, idx_node, receiver) {
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

fn find_local_array_size(source: &str, node: Node, receiver: &str) -> Option<usize> {
    if receiver.contains('.') {
        return None;
    }

    let mut cur = node;
    for _ in 0..30 {
        let Some(p) = cur.parent() else { break };

        if matches!(p.kind(), "block" | "function_item" | "source_file") {
            let mut child_cursor = p.walk();
            for child in p.children(&mut child_cursor) {
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

fn find_struct_field_array_size(
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

fn extract_impl_type_name(impl_text: &str) -> Option<&str> {
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

fn extract_field_array_size(line: &str, field_name: &str) -> Option<usize> {
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

fn find_param_array_size(source: &str, node: Node, receiver: &str) -> Option<usize> {
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_rust(code: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_rust::language()).unwrap();
        parser.parse(code, None).unwrap()
    }

    #[test]
    fn extract_receiver_simple() {
        assert_eq!(extract_receiver("v[0]"), "v");
        assert_eq!(extract_receiver("self.s[0]"), "self.s");
        assert_eq!(extract_receiver("data[0]"), "data");
    }

    #[test]
    fn fixed_array_boundary_exact() {
        // arr[3] on [u8; 4] — safe (index 3 < size 4)
        let code = r"
            fn f() {
                let arr: [u8; 4] = [0; 4];
                let _ = arr[3];
            }
        ";
        let tree = parse_rust(code);
        let root = tree.root_node();
        // Find the index_expression for arr[3]
        let idx_node = find_index_expr(&tree, code, "arr[3]");
        assert!(idx_node.is_some(), "should find arr[3] index expression");
        if let Some(node) = idx_node {
            assert!(
                is_fixed_size_array_access(code, node, root),
                "arr[3] on [u8; 4] is safe"
            );
        }
    }

    #[test]
    fn fixed_array_boundary_out_of_bounds() {
        // arr[4] on [u8; 4] — NOT safe (index 4 >= size 4)
        let code = r"
            fn f() {
                let arr: [u8; 4] = [0; 4];
                let _ = arr[4];
            }
        ";
        let tree = parse_rust(code);
        let root = tree.root_node();
        let idx_node = find_index_expr(&tree, code, "arr[4]");
        assert!(idx_node.is_some());
        if let Some(node) = idx_node {
            assert!(
                !is_fixed_size_array_access(code, node, root),
                "arr[4] on [u8; 4] must NOT be considered safe"
            );
        }
    }

    #[test]
    fn generic_impl_type_extraction() {
        let impl_text = "impl<T: Clone> MyStruct<T> {";
        let name = extract_impl_type_name(impl_text);
        assert_eq!(name, Some("MyStruct"));
    }

    #[test]
    fn trait_impl_type_extraction() {
        let impl_text = "impl Display for MyStruct {";
        let name = extract_impl_type_name(impl_text);
        assert_eq!(name, Some("MyStruct"));
    }

    #[test]
    fn pub_field_array_size() {
        assert_eq!(
            extract_field_array_size("    pub s: [u32; 4],", "s"),
            Some(4)
        );
        assert_eq!(extract_field_array_size("    s: [u8; 16],", "s"), Some(16));
        assert_eq!(extract_field_array_size("    s: Vec<u8>,", "s"), None);
    }

    /// Helper: find an index_expression node whose text matches `target`.
    fn find_index_expr<'a>(
        tree: &'a tree_sitter::Tree,
        source: &str,
        target: &str,
    ) -> Option<Node<'a>> {
        find_node_by_kind_and_text(tree.root_node(), source, "index_expression", target)
    }

    fn find_node_by_kind_and_text<'a>(
        node: Node<'a>,
        source: &str,
        kind: &str,
        target: &str,
    ) -> Option<Node<'a>> {
        if node.kind() == kind {
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");
            if text == target {
                return Some(node);
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = find_node_by_kind_and_text(child, source, kind, target) {
                return Some(found);
            }
        }
        None
    }
}
