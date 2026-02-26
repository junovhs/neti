// src/analysis/patterns/logic_helpers.rs
//! Shared helpers for L02/L03 logic pattern detection.
//!
//! Extracted from `logic.rs` to satisfy the Law of Atomicity.

use tree_sitter::Node;

#[cfg(test)]
#[path = "logic_helpers_test.rs"]
mod tests;

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

        if matches!(p.kind(), "block" | "function_item" | "source_file")
            && scope_has_let_decl(source, node, p, var_name)
        {
            return true;
        }

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

/// Scan a block/function scope for a `let` declaration matching `var_name`
/// that appears before `node`.
fn scope_has_let_decl(source: &str, node: Node, scope: Node, var_name: &str) -> bool {
    let mut child_cursor = scope.walk();
    for child in scope.children(&mut child_cursor) {
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
    false
}

/// Check if a function's parameter list contains a parameter with the given name.
pub fn has_matching_parameter(source: &str, fn_node: Node, var_name: &str) -> bool {
    let fn_text = fn_node.utf8_text(source.as_bytes()).unwrap_or("");

    let Some(paren_start) = fn_text.find('(') else {
        return false;
    };

    let Some(end) = find_matching_paren(fn_text, paren_start) else {
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
