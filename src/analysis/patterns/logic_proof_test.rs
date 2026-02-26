// src/analysis/patterns/logic_proof_test.rs

use super::*;
use tree_sitter::Parser;

fn parse_rust(code: &str) -> tree_sitter::Tree {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
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
    let code = r"
        fn f() {
            let arr: [u8; 4] = [0; 4];
            let _ = arr[3];
        }
    ";
    let tree = parse_rust(code);
    let root = tree.root_node();
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
    let name = helpers::extract_impl_type_name(impl_text);
    assert_eq!(name, Some("MyStruct"));
}

#[test]
fn trait_impl_type_extraction() {
    let impl_text = "impl Display for MyStruct {";
    let name = helpers::extract_impl_type_name(impl_text);
    assert_eq!(name, Some("MyStruct"));
}

#[test]
fn pub_field_array_size() {
    assert_eq!(
        helpers::extract_field_array_size("    pub s: [u32; 4],", "s"),
        Some(4)
    );
    assert_eq!(
        helpers::extract_field_array_size("    s: [u8; 16],", "s"),
        Some(16)
    );
    assert_eq!(
        helpers::extract_field_array_size("    s: Vec<u8>,", "s"),
        None
    );
}

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
