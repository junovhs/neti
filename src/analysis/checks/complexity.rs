// src/analysis/checks/complexity.rs
//! Complexity metrics checks (Law of Complexity).

use tree_sitter::{Node, Query, QueryCursor, QueryMatch};

use crate::config::RuleConfig;
use crate::types::{Violation, ViolationDetails};

use super::CheckContext;

/// Checks for complexity metrics (arity, depth, cyclomatic complexity).
pub fn check_metrics(
    ctx: &CheckContext,
    func_query: &Query,
    complexity_query: &Query,
    out: &mut Vec<Violation>,
) -> usize {
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(func_query, ctx.root, ctx.source.as_bytes());

    let mut max_complexity = 0;
    for m in matches {
        let comp = process_match(&m, ctx, complexity_query, out);
        if comp > max_complexity {
            max_complexity = comp;
        }
    }
    max_complexity
}

fn process_match(
    m: &QueryMatch,
    ctx: &CheckContext,
    complexity_query: &Query,
    out: &mut Vec<Violation>,
) -> usize {
    for capture in m.captures {
        let node = capture.node;
        if is_function_kind(node.kind()) {
            return analyze_function(node, ctx, complexity_query, out);
        }
    }
    0
}

fn is_function_kind(kind: &str) -> bool {
    matches!(
        kind,
        "function_item"
            | "function_definition"
            | "method_definition"
            | "arrow_function"
            | "function_declaration"
    )
}

fn analyze_function(
    node: Node,
    ctx: &CheckContext,
    complexity_query: &Query,
    out: &mut Vec<Violation>,
) -> usize {
    let row = node.start_position().row + 1;

    // Check Arity first (cheap)
    check_arity(node, ctx.config, ctx.source, row, out);
    
    // Check Nesting
    check_nesting(node, ctx.config, ctx.source, row, out);
    
    // Check Complexity
    check_cyclomatic(node, ctx, complexity_query, out)
}

// Return &str to avoid allocation (P02 fix)
fn extract_function_name<'a>(node: Node, source: &'a str) -> &'a str {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" || child.kind() == "property_identifier" {
            if let Ok(name) = child.utf8_text(source.as_bytes()) {
                return name;
            }
        }
    }
    "<anonymous>"
}

fn check_arity(
    node: Node,
    config: &RuleConfig,
    source: &str,
    row: usize,
    out: &mut Vec<Violation>,
) {
    let params = count_parameters(node);
    if params > config.max_function_args {
        let func_name = extract_function_name(node, source);
        let details = ViolationDetails {
            function_name: Some(func_name.to_string()),
            analysis: vec![format!("Function accepts {params} parameters")],
            suggestion: Some("Group related parameters into a struct or options object".into()),
        };
        out.push(Violation::with_details(
            row,
            format!("Function '{func_name}' has {params} args (Max: {})", config.max_function_args),
            "LAW OF COMPLEXITY",
            details,
        ));
    }
}

fn count_parameters(node: Node) -> usize {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "parameters" || child.kind() == "formal_parameters" {
            return child.named_child_count();
        }
    }
    0
}

fn check_nesting(
    node: Node,
    config: &RuleConfig,
    source: &str,
    row: usize,
    out: &mut Vec<Violation>,
) {
    let (depth, deepest_line) = measure_nesting(node, 0, row);
    if depth > config.max_nesting_depth {
        let func_name = extract_function_name(node, source);
        let details = ViolationDetails {
            function_name: Some(func_name.to_string()),
            analysis: vec![format!("Deepest nesting at line {deepest_line}")],
            suggestion: Some("Extract nested logic or use early returns".into()),
        };
        out.push(Violation::with_details(
            row,
            format!("Function '{func_name}' has nesting depth {depth} (Max: {})", config.max_nesting_depth),
            "LAW OF COMPLEXITY",
            details,
        ));
    }
}

fn measure_nesting(node: Node, current: usize, base_row: usize) -> (usize, usize) {
    let mut max_depth = current;
    let mut deepest_line = base_row;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let child_depth = if is_nesting_node(child.kind()) { current + 1 } else { current };
        let child_row = child.start_position().row + 1;
        let (sub_depth, sub_line) = measure_nesting(child, child_depth, child_row);
        if sub_depth > max_depth {
            max_depth = sub_depth;
            deepest_line = sub_line;
        }
    }

    (max_depth, deepest_line)
}

fn is_nesting_node(kind: &str) -> bool {
    matches!(
        kind,
        "if_expression" | "if_statement" | "for_expression" | "for_statement"
            | "for_in_statement" | "while_expression" | "while_statement"
            | "loop_expression" | "match_expression" | "switch_statement" | "try_statement"
    )
}

fn check_cyclomatic(
    node: Node,
    ctx: &CheckContext,
    query: &Query,
    out: &mut Vec<Violation>,
) -> usize {
    let (complexity, branch_lines) = measure_complexity(node, ctx.source, query);

    if complexity > ctx.config.max_cyclomatic_complexity {
        let func_name = extract_function_name(node, ctx.source);
        let row = node.start_position().row + 1;
        
        let analysis: Vec<String> = branch_lines
            .iter()
            .take(5)
            .map(|(line, kind)| format!("Branch at line {line}: {kind}"))
            .collect();

        let suggestion = if complexity > 12 {
            "Very complex. Break into smaller functions."
        } else {
            "Extract conditionals into helpers or use lookup tables."
        };

        let details = ViolationDetails {
            function_name: Some(func_name.to_string()),
            analysis,
            suggestion: Some(suggestion.to_string()),
        };

        out.push(Violation::with_details(
            row,
            format!(
                "Function '{func_name}' has cyclomatic complexity {complexity} (Max: {})",
                ctx.config.max_cyclomatic_complexity
            ),
            "LAW OF COMPLEXITY",
            details,
        ));
    }
    complexity
}

fn measure_complexity(node: Node, source: &str, query: &Query) -> (usize, Vec<(usize, String)>) {
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(query, node, source.as_bytes());

    let mut count = 1;
    let mut branches = Vec::new();

    for m in matches {
        for capture in m.captures {
            let line = capture.node.start_position().row + 1;
            let kind = capture.node.kind().to_string();
            branches.push((line, kind));
            count += 1;
        }
    }

    (count, branches)
}