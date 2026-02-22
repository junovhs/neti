//! Shared helpers for L02/L03 logic pattern detection.
//!
//! Extracted from `logic.rs` to satisfy the Law of Atomicity.

use tree_sitter::Node;

/// Returns `true` if the node is an integer or float literal.
pub fn is_literal(node: Option<Node>) -> bool {
    node.is_some_and(|n| n.kind() == "integer_literal" || n.kind() == "float_literal")
}

/// Returns `true` if the name looks like a loop index variable.
pub fn is_index_variable(name: &str) -> bool {
    let n = name.trim();
    n == "i"
        || n == "j"
        || n == "k"
        || n == "n"
        || n == "idx"
        || n.contains("index")
        || n.contains("pos")
        || n.contains("ptr")
        || n.contains("offset")
        || n.contains("cursor")
}

/// Returns `true` if a parent scope contains a `.len()` or `.is_empty()` guard.
pub fn has_explicit_guard(source: &str, node: Node) -> bool {
    let mut cur = node;
    for _ in 0..10 {
        let Some(p) = cur.parent() else { break };
        let kind = p.kind();
        let text = p.utf8_text(source.as_bytes()).unwrap_or("");
        if text.contains(".len()") || text.contains(".is_empty()") {
            return true;
        }
        if kind == "if_expression" && text.contains('!') && text.contains("is_empty") {
            return true;
        }
        cur = p;
    }
    false
}

/// Returns `true` if the node is inside a `chunks_exact(N)` or `array_chunks()` iterator.
pub fn has_chunks_exact_context(source: &str, node: Node) -> bool {
    let mut cur = node;
    for _ in 0..25 {
        let Some(p) = cur.parent() else { break };
        let text = p.utf8_text(source.as_bytes()).unwrap_or("");
        if text.contains("chunks_exact(") || text.contains("array_chunks(") {
            return true;
        }
        if p.kind() == "source_file" {
            break;
        }
        cur = p;
    }
    false
}

/// Check whether a `let` declaration matches `var_name`.
///
/// Handles `let x`, `let mut x`, with delimiters ` `, `:`, `=`, `;`.
pub fn decl_matches_variable(decl_text: &str, var_name: &str) -> bool {
    let after_let = decl_text.strip_prefix("let").unwrap_or(decl_text).trim();
    let after_mut = after_let.strip_prefix("mut").unwrap_or(after_let).trim();
    after_mut.starts_with(var_name) && after_mut[var_name.len()..].starts_with([' ', ':', '=', ';'])
}

/// Check whether a `let` declaration for `var_name` exists in any enclosing scope,
/// or whether `var_name` is a function/closure parameter.
pub fn can_find_local_declaration(source: &str, node: Node, var_name: &str) -> bool {
    if var_name.contains('.') {
        return false;
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
                if decl_matches_variable(decl_text, var_name) {
                    return true;
                }
            }
        }

        // Check function parameters
        if p.kind() == "function_item" {
            if has_matching_parameter(source, p, var_name) {
                return true;
            }
            break;
        }

        if p.kind() == "source_file" {
            break;
        }

        cur = p;
    }
    false
}

/// Check if a function's parameter list contains a parameter with the given name.
pub fn has_matching_parameter(source: &str, fn_node: Node, var_name: &str) -> bool {
    let fn_text = fn_node.utf8_text(source.as_bytes()).unwrap_or("");

    let Some(paren_start) = fn_text.find('(') else {
        return false;
    };

    let paren_end = find_matching_paren(fn_text, paren_start);

    let Some(end) = paren_end else {
        return false;
    };

    let params = &fn_text[paren_start + 1..end];
    params_contain_name(params, var_name)
}

/// Find the closing `)` that matches the `(` at `start`.
fn find_matching_paren(text: &str, start: usize) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in text[start..].char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Check if a comma-separated parameter list contains `var_name`.
fn params_contain_name(params: &str, var_name: &str) -> bool {
    for param in params.split(',') {
        let trimmed = param.trim();
        let clean = trimmed
            .strip_prefix("mut ")
            .or_else(|| trimmed.strip_prefix("&mut "))
            .or_else(|| trimmed.strip_prefix('&'))
            .unwrap_or(trimmed)
            .trim();

        if let Some(colon_pos) = clean.find(':') {
            let param_name = clean[..colon_pos].trim();
            if param_name == var_name {
                return true;
            }
        }
    }
    false
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

    // ── is_literal ──────────────────────────────────────────────────────

    #[test]
    fn is_literal_detects_integer() {
        let code = "fn f() { 42 }";
        let tree = parse_rust(code);
        let root = tree.root_node();
        // Walk to find the integer_literal
        let mut found = false;
        let mut cursor = root.walk();
        visit_all(root, &mut cursor, &mut |n| {
            if n.kind() == "integer_literal" {
                assert!(is_literal(Some(n)));
                found = true;
            }
        });
        assert!(found, "should find an integer literal");
    }

    #[test]
    fn is_literal_rejects_identifier() {
        let code = "fn f(x: i32) { x }";
        let tree = parse_rust(code);
        let root = tree.root_node();
        let mut cursor = root.walk();
        visit_all(root, &mut cursor, &mut |n| {
            if n.kind() == "identifier" {
                assert!(!is_literal(Some(n)));
            }
        });
    }

    #[test]
    fn is_literal_none_returns_false() {
        assert!(!is_literal(None));
    }

    // ── decl_matches_variable ───────────────────────────────────────────

    #[test]
    fn decl_matches_exact_name() {
        assert!(decl_matches_variable("let v = vec![];", "v"));
        assert!(decl_matches_variable("let mut v: Vec<i32> = vec![];", "v"));
    }

    #[test]
    fn decl_no_false_prefix_match() {
        // "values" starts with "v" but is not "v"
        assert!(!decl_matches_variable("let values = vec![];", "v"));
        assert!(!decl_matches_variable("let v2 = 0;", "v"));
    }

    // ── has_matching_parameter ──────────────────────────────────────────

    #[test]
    fn matches_plain_param() {
        let code = "fn f(v: &[i32]) { }";
        let tree = parse_rust(code);
        let root = tree.root_node();
        let fn_node = root.child(0).unwrap();
        assert!(has_matching_parameter(code, fn_node, "v"));
        assert!(!has_matching_parameter(code, fn_node, "x"));
    }

    #[test]
    fn matches_mut_param() {
        let code = "fn f(mut buf: Vec<u8>) { }";
        let tree = parse_rust(code);
        let fn_node = tree.root_node().child(0).unwrap();
        assert!(has_matching_parameter(code, fn_node, "buf"));
    }

    #[test]
    fn matches_ref_mut_param() {
        let code = "fn f(data: &mut [u8]) { }";
        let tree = parse_rust(code);
        let fn_node = tree.root_node().child(0).unwrap();
        assert!(has_matching_parameter(code, fn_node, "data"));
    }

    // ── helper for visiting all nodes ───────────────────────────────────

    fn visit_all<'a>(
        node: tree_sitter::Node<'a>,
        cursor: &mut tree_sitter::TreeCursor<'a>,
        f: &mut dyn FnMut(tree_sitter::Node<'a>),
    ) {
        f(node);
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                let mut child_cursor = child.walk();
                visit_all(child, &mut child_cursor, f);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }
}
