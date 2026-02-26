//! Test context detection for pattern detectors.
//!
//! Extracted from `performance.rs` to satisfy the Law of Atomicity
//! and the Law of Complexity (cognitive complexity was 31).

use tree_sitter::Node;

/// Returns `true` if the node is inside a `#[test]` function or `#[cfg(test)]` module.
pub fn is_test_context(source: &str, node: Node) -> bool {
    let mut cur = node;
    for _ in 0..30 {
        let Some(p) = cur.parent() else { break };

        match p.kind() {
            "function_item" => {
                if has_test_attribute(source, p) {
                    return true;
                }
            }
            "mod_item" => {
                if has_cfg_test_attribute(source, p) {
                    return true;
                }
            }
            "source_file" => break,
            _ => {}
        }

        cur = p;
    }
    false
}

/// Checks if a function node has a `#[test]` attribute by examining
/// the AST for attribute_item siblings immediately preceding it.
fn has_test_attribute(source: &str, fn_node: Node) -> bool {
    let Some(parent) = fn_node.parent() else {
        return false;
    };

    let mut cursor = parent.walk();
    let mut prev_was_test = false;

    for child in parent.children(&mut cursor) {
        if child.id() == fn_node.id() && prev_was_test {
            return true;
        }

        prev_was_test = is_test_attr_node(source, &child);
    }

    false
}

/// Returns `true` if `node` is an `attribute_item` containing `#[test]`.
fn is_test_attr_node(source: &str, node: &Node) -> bool {
    if node.kind() != "attribute_item" {
        return false;
    }
    let text = node.utf8_text(source.as_bytes()).unwrap_or("");
    text.contains("#[test]")
}

/// Checks if a module node has a `#[cfg(test)]` attribute.
fn has_cfg_test_attribute(source: &str, mod_node: Node) -> bool {
    let Some(parent) = mod_node.parent() else {
        return false;
    };

    let mut cursor = parent.walk();
    let mut prev_was_cfg_test = false;

    for child in parent.children(&mut cursor) {
        if child.id() == mod_node.id() && prev_was_cfg_test {
            return true;
        }

        prev_was_cfg_test = is_cfg_test_attr_node(source, &child);
    }

    false
}

/// Returns `true` if `node` is an `attribute_item` containing `#[cfg(test)]`.
fn is_cfg_test_attr_node(source: &str, node: &Node) -> bool {
    if node.kind() != "attribute_item" {
        return false;
    }
    let text = node.utf8_text(source.as_bytes()).unwrap_or("");
    text.contains("#[cfg(test)]")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_rust(code: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
        parser.parse(code, None).unwrap()
    }

    fn find_first_node_by_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
        if node.kind() == kind {
            return Some(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = find_first_node_by_kind(child, kind) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn detects_test_function() {
        let code = r#"
            #[test]
            fn test_something() {
                let x = 1;
            }
        "#;
        let tree = parse_rust(code);
        let root = tree.root_node();
        let let_node =
            find_first_node_by_kind(root, "let_declaration").expect("should find let_declaration");
        assert!(
            is_test_context(code, let_node),
            "node inside #[test] fn should be detected"
        );
    }

    #[test]
    fn detects_cfg_test_module() {
        let code = r#"
            #[cfg(test)]
            mod tests {
                fn helper() {
                    let x = 1;
                }
            }
        "#;
        let tree = parse_rust(code);
        let root = tree.root_node();
        let let_node =
            find_first_node_by_kind(root, "let_declaration").expect("should find let_declaration");
        assert!(
            is_test_context(code, let_node),
            "node inside #[cfg(test)] mod should be detected"
        );
    }

    #[test]
    fn non_test_context() {
        let code = r"
            fn regular_function() {
                let x = 1;
            }
        ";
        let tree = parse_rust(code);
        let root = tree.root_node();
        let let_node =
            find_first_node_by_kind(root, "let_declaration").expect("should find let_declaration");
        assert!(
            !is_test_context(code, let_node),
            "node in regular fn should NOT be detected as test context"
        );
    }
}
