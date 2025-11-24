use crate::config::RuleConfig;
use crate::metrics;
use crate::types::Violation;
use anyhow::Result;
use tree_sitter::{Node, Query, QueryCursor};

/// Context object to solve "High Arity" issues.
pub struct CheckContext<'a> {
    pub root: Node<'a>,
    pub source: &'a str,
    pub filename: &'a str,
    pub config: &'a RuleConfig,
}

// --- LAW OF BLUNTNESS ---

/// Checks for function naming violations.
///
/// # Errors
///
/// Currently always returns `Ok` as violations are collected in the mutable vector.
/// The `Result` type is preserved for future extensibility (e.g., query failure propagation).
#[allow(clippy::unnecessary_wraps)]
pub fn check_naming(ctx: &CheckContext, query: &Query, out: &mut Vec<Violation>) -> Result<()> {
    let mut cursor = QueryCursor::new();
    for m in cursor.matches(query, ctx.root, ctx.source.as_bytes()) {
        let node = m.captures[0].node;
        let name = node.utf8_text(ctx.source.as_bytes()).unwrap_or("?");

        let word_count = if name.contains('_') {
            name.split('_').count()
        } else {
            let cap_count = name.chars().filter(|c| c.is_uppercase()).count();
            if name.chars().next().is_some_and(char::is_uppercase) {
                cap_count
            } else {
                cap_count + 1
            }
        };

        let should_ignore = ctx
            .config
            .ignore_naming_on
            .iter()
            .any(|p| ctx.filename.contains(p));

        if word_count > ctx.config.max_function_words && !should_ignore {
            out.push(Violation {
                row: node.start_position().row,
                message: format!(
                    "Function '{name}' has {word_count} words (Max: {}). Is it doing too much?",
                    ctx.config.max_function_words
                ),
                law: "LAW OF BLUNTNESS",
            });
        }
    }
    Ok(())
}

// --- LAW OF PARANOIA ---

/// Checks for structural safety in logic blocks.
///
/// # Errors
///
/// Currently always returns `Ok`. The `Result` return type is preserved for architectural
/// consistency.
#[allow(clippy::unnecessary_wraps)]
pub fn check_safety(
    ctx: &CheckContext,
    safety_query: &Query,
    out: &mut Vec<Violation>,
) -> Result<()> {
    let mut cursor = ctx.root.walk();
    loop {
        let node = cursor.node();
        let kind = node.kind();

        if (kind.contains("function") || kind.contains("method")) && !is_lifecycle(node, ctx.source)
        {
            let rows = node.end_position().row - node.start_position().row;
            if rows > 5 {
                let mut func_cursor = QueryCursor::new();
                if func_cursor
                    .matches(safety_query, node, ctx.source.as_bytes())
                    .next()
                    .is_none()
                {
                    out.push(Violation {
                        row: node.start_position().row,
                        message:
                            "Function lacks explicit error handling (Result, match, try/catch)."
                                .into(),
                        law: "LAW OF PARANOIA",
                    });
                }
            }
        }

        if !cursor.goto_first_child() {
            while !cursor.goto_next_sibling() {
                if !cursor.goto_parent() {
                    return Ok(());
                }
            }
        }
    }
}

// --- LAW OF COMPLEXITY ---

/// Checks for complexity metrics (Arity, Depth, Cyclomatic Complexity).
///
/// # Errors
///
/// Currently always returns `Ok`. The `Result` return type is preserved for architectural
/// consistency.
#[allow(clippy::unnecessary_wraps)]
pub fn check_metrics(
    ctx: &CheckContext,
    complexity_query: &Query,
    out: &mut Vec<Violation>,
) -> Result<()> {
    let mut cursor = ctx.root.walk();
    loop {
        let node = cursor.node();
        let kind = node.kind();

        if kind.contains("function") || kind.contains("method") {
            // 1. Check Arity
            let args = metrics::count_arguments(node);
            if args > ctx.config.max_function_args {
                out.push(Violation {
                    row: node.start_position().row,
                    message: format!(
                        "High Arity: Function takes {args} arguments (Max: {}). Use a Struct.",
                        ctx.config.max_function_args
                    ),
                    law: "LAW OF COMPLEXITY",
                });
            }

            // 2. Check Nesting Depth
            let depth = metrics::calculate_max_depth(node);
            if depth > ctx.config.max_nesting_depth {
                out.push(Violation {
                    row: node.start_position().row,
                    message: format!(
                        "Deep Nesting: Max depth is {depth} (Max: {}). Extract logic.",
                        ctx.config.max_nesting_depth
                    ),
                    law: "LAW OF COMPLEXITY",
                });
            }

            // 3. Check Cyclomatic Complexity
            let complexity = metrics::calculate_complexity(node, ctx.source, complexity_query);
            if complexity > ctx.config.max_cyclomatic_complexity {
                out.push(Violation {
                    row: node.start_position().row,
                    message: format!(
                        "High Complexity: Score is {complexity} (Max: {}). Hard to test.",
                        ctx.config.max_cyclomatic_complexity
                    ),
                    law: "LAW OF COMPLEXITY",
                });
            }
        }

        if !cursor.goto_first_child() {
            while !cursor.goto_next_sibling() {
                if !cursor.goto_parent() {
                    return Ok(());
                }
            }
        }
    }
}

/// Checks for banned constructs like `unwrap`.
///
/// # Errors
///
/// Currently always returns `Ok`. The `Result` return type is preserved for architectural
/// consistency.
#[allow(clippy::unnecessary_wraps)]
pub fn check_banned(
    ctx: &CheckContext,
    banned_query: &Query,
    out: &mut Vec<Violation>,
) -> Result<()> {
    let mut cursor = QueryCursor::new();
    for m in cursor.matches(banned_query, ctx.root, ctx.source.as_bytes()) {
        let node = m.captures[0].node;
        out.push(Violation {
            row: node.start_position().row,
            message: "Explicit 'unwrap()' call detected. Use 'expect', 'unwrap_or', or '?'.".into(),
            law: "LAW OF PARANOIA",
        });
    }
    Ok(())
}

fn is_lifecycle(node: Node, source: &str) -> bool {
    if let Some(name_node) = node.child_by_field_name("name") {
        let name = name_node.utf8_text(source.as_bytes()).unwrap_or("");
        return matches!(
            name,
            "new" | "default" | "init" | "__init__" | "constructor" | "render" | "main"
        );
    }
    false
}
