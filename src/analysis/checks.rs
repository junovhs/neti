// src/analysis/checks.rs
use crate::config::types::RuleConfig;
use crate::types::Violation;
use std::path::Path;
use tree_sitter::{Node, Query, QueryCursor, QueryMatch};

pub struct CheckContext<'a> {
    pub root: Node<'a>,
    pub source: &'a str,
    pub filename: &'a str,
    pub config: &'a RuleConfig,
}

/// Checks for naming violations (function name word count).
pub fn check_naming(ctx: &CheckContext, query: &Query, out: &mut Vec<Violation>) {
    for pattern in &ctx.config.ignore_naming_on {
        if ctx.filename.contains(pattern) {
            return;
        }
    }

    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(query, ctx.root, ctx.source.as_bytes());

    for m in matches {
        process_naming_match(&m, ctx, out);
    }
}

fn process_naming_match(m: &QueryMatch, ctx: &CheckContext, out: &mut Vec<Violation>) {
    for capture in m.captures {
        if let Ok(name) = capture.node.utf8_text(ctx.source.as_bytes()) {
            let word_count = count_words(name);
            if word_count > ctx.config.max_function_words {
                out.push(Violation {
                    row: capture.node.start_position().row + 1,
                    message: format!(
                        "Function name '{name}' has {word_count} words (Max: {})",
                        ctx.config.max_function_words
                    ),
                    law: "LAW OF CONCISENESS",
                });
            }
        }
    }
}

/// Checks for complexity metrics (arity, depth, cyclomatic complexity).
pub fn check_metrics(
    ctx: &CheckContext,
    func_query: &Query,
    complexity_query: &Query,
    out: &mut Vec<Violation>,
) {
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(func_query, ctx.root, ctx.source.as_bytes());

    for m in matches {
        process_metric_match(&m, ctx, complexity_query, out);
    }
}

fn process_metric_match(
    m: &QueryMatch,
    ctx: &CheckContext,
    complexity_query: &Query,
    out: &mut Vec<Violation>,
) {
    for capture in m.captures {
        let node = capture.node;
        if is_function_kind(node.kind()) {
            analyze_function_node(node, ctx, complexity_query, out);
            return;
        }
    }
}

fn analyze_function_node(
    node: Node,
    ctx: &CheckContext,
    complexity_query: &Query,
    out: &mut Vec<Violation>,
) {
    let func_name = get_function_name(node, ctx.source);

    check_argument_count(node, &func_name, ctx.config, out);

    if let Some(body) = node.child_by_field_name("body") {
        check_nesting_depth(node, body, ctx.config, out);
        // Pass ctx directly to avoid 6 arguments
        check_cyclomatic_complexity(node, body, ctx, complexity_query, out);
    }
}

fn is_function_kind(kind: &str) -> bool {
    kind.contains("function") || kind.contains("method")
}

fn get_function_name(node: Node, source: &str) -> String {
    node.child_by_field_name("name")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("<anon>")
        .to_string()
}

fn check_argument_count(
    node: Node,
    name: &str,
    config: &RuleConfig,
    out: &mut Vec<Violation>,
) {
    let arg_count = super::metrics::count_arguments(node);
    if arg_count > config.max_function_args {
        out.push(Violation {
            row: node.start_position().row + 1,
            message: format!(
                "Function '{name}' has {arg_count} args (Max: {})",
                config.max_function_args
            ),
            law: "LAW OF COMPLEXITY",
        });
    }
}

fn check_nesting_depth(
    func_node: Node,
    body: Node,
    config: &RuleConfig,
    out: &mut Vec<Violation>,
) {
    let depth = super::metrics::calculate_max_depth(body);
    if depth > config.max_nesting_depth {
        out.push(Violation {
            row: func_node.start_position().row + 1,
            message: format!(
                "Deep Nesting: Max depth is {depth}. Extract logic. (Max: {})",
                config.max_nesting_depth
            ),
            law: "LAW OF COMPLEXITY",
        });
    }
}

fn check_cyclomatic_complexity(
    func_node: Node,
    body: Node,
    ctx: &CheckContext,
    query: &Query,
    out: &mut Vec<Violation>,
) {
    let complexity = super::metrics::calculate_complexity(body, ctx.source, query);
    if complexity > ctx.config.max_cyclomatic_complexity {
        out.push(Violation {
            row: func_node.start_position().row + 1,
            message: format!(
                "High Complexity: Score is {complexity}. Hard to test. (Max: {})",
                ctx.config.max_cyclomatic_complexity
            ),
            law: "LAW OF COMPLEXITY",
        });
    }
}

/// Checks for banned constructs (`.unwrap()` and `.expect()` calls).
pub fn check_banned(ctx: &CheckContext, banned_query: &Query, out: &mut Vec<Violation>) {
    // Only skip if the FILE NAME indicates a test, not the directory path.
    // This fixes integration tests running in temporary directories.
    let path = Path::new(ctx.filename);
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if name.contains("test") || name.contains("spec") {
            return;
        }
    }

    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(banned_query, ctx.root, ctx.source.as_bytes());

    for m in matches {
        process_banned_match_group(&m, ctx, out);
    }
}

fn process_banned_match_group(m: &QueryMatch, ctx: &CheckContext, out: &mut Vec<Violation>) {
    for capture in m.captures {
        if let Ok(text) = capture.node.utf8_text(ctx.source.as_bytes()) {
            let row = capture.node.start_position().row + 1;
            let kind = capture.node.kind();
            if kind == "method_invocation"
                || kind == "call_expression"
                || kind == "method_call_expression"
                || text.contains("unwrap")
                || text.contains("expect")
            {
                add_banned_violation(text, row, out);
            }
        }
    }
}

fn add_banned_violation(text: &str, row: usize, out: &mut Vec<Violation>) {
    if text.contains("unwrap") {
        out.push(Violation {
            row,
            message: "Banned: '.unwrap()' found. Use ? or expect().".to_string(),
            law: "LAW OF PARANOIA",
        });
    } else if text.contains("expect") {
        out.push(Violation {
            row,
            message: "Banned: '.expect()' found. Use handleable errors.".to_string(),
            law: "LAW OF PARANOIA",
        });
    }
}

/// Checks for unsafe blocks and ensures they have justification comments.
pub fn check_safety(ctx: &CheckContext, _query: &Query, out: &mut Vec<Violation>) {
    if !Path::new(ctx.filename)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"))
    {
        return;
    }
    traverse_for_unsafe(ctx.root, ctx.source, ctx.config, out);
}

fn traverse_for_unsafe(
    node: Node,
    source: &str,
    config: &RuleConfig,
    out: &mut Vec<Violation>,
) {
    if node.kind() == "unsafe_block" {
        validate_unsafe_node(node, source, config, out);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        traverse_for_unsafe(child, source, config, out);
    }
}

fn validate_unsafe_node(
    node: Node,
    source: &str,
    config: &RuleConfig,
    out: &mut Vec<Violation>,
) {
    if config.safety.ban_unsafe {
        out.push(Violation {
            row: node.start_position().row + 1,
            message: "Unsafe code is strictly prohibited by configuration.".to_string(),
            law: "LAW OF PARANOIA",
        });
        return;
    }

    if config.safety.require_safety_comment && !has_safety_comment(node, source) {
        out.push(Violation {
            row: node.start_position().row + 1,
            message: "Unsafe block missing justification. Add '// SAFETY:' comment.".to_string(),
            law: "LAW OF PARANOIA",
        });
    }
}

fn has_safety_comment(node: Node, source: &str) -> bool {
    let mut prev = node.prev_sibling();
    while let Some(p) = prev {
        if p.kind() == "line_comment" || p.kind() == "block_comment" {
            if let Ok(text) = p.utf8_text(source.as_bytes()) {
                if text.contains("SAFETY:") {
                    return true;
                }
            }
        }
        if p.kind() != "line_comment" && p.kind() != "block_comment" {
            return false;
        }
        prev = p.prev_sibling();
    }
    check_lines_above(node.start_position().row, source)
}

fn check_lines_above(row: usize, source: &str) -> bool {
    if row == 0 {
        return false;
    }
    let lines: Vec<&str> = source.lines().collect();
    for i in 1..=3 {
        if row < i {
            break;
        }
        if let Some(line) = lines.get(row - i) {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with("//") && trimmed.contains("SAFETY:") {
                return true;
            }
            return false;
        }
    }
    false
}

fn count_words(name: &str) -> usize {
    name.split('_').filter(|part| !part.is_empty()).count()
}