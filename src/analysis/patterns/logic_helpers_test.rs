// src/analysis/patterns/logic_helpers_test.rs

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
