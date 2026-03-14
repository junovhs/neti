//! Test context detection for pattern detectors.
//!
//! Extracted from `performance.rs` to satisfy the Law of Atomicity
//! and the Law of Complexity (cognitive complexity was 31).

use tree_sitter::Node;
use omni_ast::{semantics_for, LangSemantics, SemanticContext, SemanticLanguage};

/// Returns `true` if the node is inside a `#[test]` function or `#[cfg(test)]` module.
pub fn is_test_context(source: &str, node: Node, language: SemanticLanguage) -> bool {
    let mut cur = node;
    let semantics = semantics_for(language);
    for _ in 0..30 {
        let Some(p) = cur.parent() else { break };

        match p.kind() {
            "function_item" | "mod_item" => {
                if semantics.is_test_context(&context_for_node(source, p)) {
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

fn context_for_node(source: &str, node: Node) -> SemanticContext {
    SemanticContext::from_source(node_with_leading_attributes(source, node))
}

fn node_with_leading_attributes(source: &str, node: Node) -> String {
    let Some(parent) = node.parent() else {
        return node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
    };

    let mut collected = Vec::new();
    let mut cursor = parent.walk();
    let mut pending_attrs = Vec::new();

    for child in parent.children(&mut cursor) {
        if child.kind() == "attribute_item" {
            pending_attrs.push(child);
            continue;
        }

        if child.id() == node.id() {
            collected.extend(pending_attrs);
            collected.push(child);
            break;
        }

        pending_attrs.clear();
    }

    if collected.is_empty() {
        return node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
    }

    let mut text = String::new();
    for part in collected {
        if let Ok(snippet) = part.utf8_text(source.as_bytes()) {
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(snippet);
        }
    }
    text
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_rust(code: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
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
            is_test_context(code, let_node, SemanticLanguage::Rust),
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
            is_test_context(code, let_node, SemanticLanguage::Rust),
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
            !is_test_context(code, let_node, SemanticLanguage::Rust),
            "node in regular fn should NOT be detected as test context"
        );
    }
}
