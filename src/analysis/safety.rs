// src/analysis/safety.rs
//! Safety checks and hygiene enforcement (Law of Paranoia).

use crate::config::types::RuleConfig;
use crate::types::Violation;
use std::path::Path;
use tree_sitter::{Node, Query};

pub struct CheckContext<'a> {
    pub root: Node<'a>,
    pub source: &'a str,
    pub filename: &'a str,
    pub config: &'a RuleConfig,
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
    // Check preceding sibling comments
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
    
    // Fallback: Check raw lines above
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