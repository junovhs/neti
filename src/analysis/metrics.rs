// src/analysis/metrics.rs
use tree_sitter::{Node, Query, QueryCursor};

/// Calculates the nesting depth of a node.
#[must_use]
pub fn calculate_max_depth(node: Node) -> usize {
    let mut max_depth = 0;
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind().contains("block") || child.kind().contains("body") {
            max_depth = std::cmp::max(max_depth, walk_depth(child, 0));
        }
    }
    max_depth
}

fn walk_depth(node: Node, current: usize) -> usize {
    let mut max = current;
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        if matches!(
            kind,
            "if_expression"
                | "match_expression"
                | "for_expression"
                | "while_expression"
                | "loop_expression"
                | "if_statement"
                | "for_statement"
                | "for_in_statement"
                | "while_statement"
                | "do_statement"
                | "switch_case"
                | "catch_clause"
                | "try_statement"
                | "closure_expression" // Rust closures
                | "arrow_function" // JS/TS
                | "function_expression" // JS/TS
                | "lambda" // Python
        ) {
            max = std::cmp::max(max, walk_depth(child, current + 1));
        } else {
            max = std::cmp::max(max, walk_depth(child, current));
        }
    }
    max
}

/// Calculates `McCabe` Cyclomatic Complexity.
#[must_use]
pub fn calculate_complexity(node: Node, source: &str, query: &Query) -> usize {
    let mut cursor = QueryCursor::new();
    let mut complexity = 1;
    for _ in cursor.matches(query, node, source.as_bytes()) {
        complexity += 1;
    }
    complexity
}

/// Counts named arguments/parameters.
#[must_use]
pub fn count_arguments(node: Node) -> usize {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind().contains("parameter") || child.kind().contains("argument") {
            return child.named_child_count();
        }
    }
    0
}