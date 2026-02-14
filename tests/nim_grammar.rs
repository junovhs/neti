//! Nim grammar validation tests for Issue [9].
//!
//! These tests validate that tree-sitter-nim compiles correctly and
//! discover the actual AST node names for query construction.
//!
//! Run with `cargo test nim_grammar -- --nocapture` to see AST output.

#![allow(clippy::unwrap_used)]

use tree_sitter::{Parser, Query};

/// Test that tree-sitter-nim compiles and links correctly.
#[test]
fn test_nim_grammar_loads() {
    let lang = tree_sitter_nim::language();
    let mut parser = Parser::new();

    let result = parser.set_language(lang);
    assert!(
        result.is_ok(),
        "Failed to set Nim language: {:?}",
        result.err()
    );
}

/// Test basic parsing of a Nim source file.
#[test]
fn test_nim_parse_basic() {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_nim::language()).unwrap();

    let source = r#"
import std/strutils

type
  User = object
    name: string
    age: int

proc greet(user: User): string =
  result = "Hello, " & user.name
"#;

    let tree = parser.parse(source, None);
    assert!(tree.is_some(), "Failed to parse Nim source");

    let tree = tree.unwrap();
    let root = tree.root_node();
    assert!(!root.has_error(), "Parse error in Nim source");
}

/// Discover actual AST node names by printing the tree structure.
/// This is critical for writing correct tree-sitter queries.
#[test]
fn test_nim_grammar_node_discovery() {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_nim::language()).unwrap();

    let source = r#"
# Test file covering major Nim constructs
import std/strutils
from os import paramCount
include helpers

type
  Color = enum
    Red, Green, Blue

  Point = object
    x, y: int

  Node* = ref object
    data: string
    next: ptr Node

const
  MaxSize* = 100

let globalVar* = 42

proc regularProc(a, b: int): string =
  if a > b:
    result = "yes"
  else:
    result = "no"

func pureFunc(x: float): float =
  x * 2.0

method virtualMethod(self: Point) {.base.} =
  discard

iterator items*(s: string): char =
  for c in s:
    yield c

template withFile(f, filename, body) =
  var f = open(filename)
  try:
    body
  finally:
    close(f)

macro simpleMacro(x: untyped): untyped =
  result = x

converter toInt*(x: float): int =
  int(x)

proc unsafeExample() =
  var x: int = 42
  let y = cast[float](x)
  let p = addr(x)

  {.push boundChecks:off.}
  var arr: array[5, int]
  echo arr[10]
  {.pop.}

  {.emit: "printf(\"hello\");".}

  asm """
    nop
  """

proc withExport*(a: int): int =
  a + 1
"#;

    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    // Print full AST for inspection
    eprintln!("\n=== NIM AST NODE DISCOVERY ===\n");
    print_tree(root, source, 0);
    eprintln!("\n=== END AST ===\n");

    // Basic sanity check
    assert!(root.child_count() > 0, "AST should have children");
}

/// Print tree structure recursively for node discovery.
fn print_tree(node: tree_sitter::Node, source: &str, indent: usize) {
    let spacing = "  ".repeat(indent);
    let kind = node.kind();

    // Show text for leaf nodes
    let text_preview = if node.child_count() == 0 {
        let node_text = node.utf8_text(source.as_bytes()).unwrap_or("?");
        let truncated: String = node_text.chars().take(40).collect();
        if truncated.len() < node_text.len() {
            format!(" {truncated:?}...")
        } else {
            format!(" {node_text:?}")
        }
    } else {
        String::new()
    };

    eprintln!("{spacing}{kind}{text_preview}");

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            print_tree(child, source, indent + 1);
        }
    }
}

/// Test that basic query patterns compile.
/// These are GUESSES based on typical tree-sitter patterns.
/// Update node names based on `test_nim_grammar_node_discovery` output.
#[test]
fn test_nim_query_compilation_naming() {
    let lang = tree_sitter_nim::language();

    // Try common patterns for proc/function declarations
    let test_queries = [
        "(proc_declaration name: (identifier) @name)",
        "(func_declaration name: (identifier) @name)",
        "(method_declaration name: (identifier) @name)",
        "(iterator_declaration name: (identifier) @name)",
        "(template_declaration name: (identifier) @name)",
        "(macro_declaration name: (identifier) @name)",
        "(converter_declaration name: (identifier) @name)",
        "(type_declaration name: (identifier) @name)",
    ];

    for query_str in test_queries {
        let result = Query::new(lang, query_str);
        if result.is_err() {
            eprintln!("Query failed (expected - will update): {query_str}");
        }
    }

    // At minimum, the language should be queryable
    let basic_query = "(_) @node";
    let result = Query::new(lang, basic_query);
    assert!(result.is_ok(), "Basic query should compile");
}

/// Test complexity-related node detection.
#[test]
fn test_nim_query_compilation_complexity() {
    let lang = tree_sitter_nim::language();

    let test_queries = [
        "(if_statement) @branch",
        "(elif_branch) @branch",
        "(else_branch) @branch",
        "(case_statement) @branch",
        "(of_branch) @branch",
        "(for_statement) @branch",
        "(while_statement) @branch",
        "(try_statement) @branch",
        "(except_branch) @branch",
        "(binary_expression) @branch",
    ];

    for query_str in test_queries {
        let result = Query::new(lang, query_str);
        if result.is_err() {
            eprintln!("Complexity query failed: {query_str}");
        }
    }
}

/// Test import statement detection.
#[test]
fn test_nim_query_compilation_imports() {
    let lang = tree_sitter_nim::language();

    let test_queries = [
        "(import_statement) @import",
        "(from_statement) @import",
        "(include_statement) @include",
    ];

    for query_str in test_queries {
        let result = Query::new(lang, query_str);
        if result.is_err() {
            eprintln!("Import query failed: {query_str}");
        }
    }
}

/// Test unsafe construct detection patterns.
#[test]
fn test_nim_query_compilation_unsafe() {
    let lang = tree_sitter_nim::language();

    let test_queries = [
        "(cast_expression) @cast",
        "(asm_statement) @asm",
        "(ptr_type) @ptr",
        "(pragma_expression) @pragma",
    ];

    for query_str in test_queries {
        let result = Query::new(lang, query_str);
        if result.is_err() {
            eprintln!("Unsafe query failed: {query_str}");
        }
    }
}
