use crate::analysis::checks;
use crate::config::RuleConfig;
use crate::types::Violation;
use std::path::Path;
use tree_sitter::{Node, Query};

/// Checks for unsafe blocks and ensures they have justification comments.
pub fn check_safety(ctx: &checks::CheckContext, _query: &Query, out: &mut Vec<Violation>) {
    if !is_rust_file(ctx.filename) {
        return;
    }
    traverse_for_unsafe(ctx.root, ctx.source, ctx.config, out);
}

fn is_rust_file(filename: &str) -> bool {
    Path::new(filename)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"))
}

fn traverse_for_unsafe(node: Node, source: &str, config: &RuleConfig, out: &mut Vec<Violation>) {
    if node.kind() == "unsafe_block" {
        validate_unsafe_node(node, source, config, out);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        traverse_for_unsafe(child, source, config, out);
    }
}

fn validate_unsafe_node(node: Node, source: &str, config: &RuleConfig, out: &mut Vec<Violation>) {
    if config.safety.ban_unsafe {
        report_ban_violation(node, out);
    } else if config.safety.require_safety_comment && !has_safety_comment(node, source) {
        report_comment_violation(node, out);
    }
}

fn report_ban_violation(node: Node, out: &mut Vec<Violation>) {
    let row = node.start_position().row + 1;
    let msg = "Unsafe code is strictly prohibited by configuration.";
    out.push(Violation::simple(row, msg.to_string(), "LAW OF PARANOIA"));
}

fn report_comment_violation(node: Node, out: &mut Vec<Violation>) {
    let row = node.start_position().row + 1;
    let msg = "Unsafe block missing justification. Add '// SAFETY:' comment.";
    out.push(Violation::simple(row, msg.to_string(), "LAW OF PARANOIA"));
}

fn has_safety_comment(node: Node, source: &str) -> bool {
    check_sibling_comments(node, source) || check_lines_above(node.start_position().row, source)
}

fn check_sibling_comments(node: Node, source: &str) -> bool {
    let mut prev = node.prev_sibling();
    while let Some(p) = prev {
        if check_single_sibling(p, source) {
            return true;
        }
        if !is_comment_node(p.kind()) {
            return false;
        }
        prev = p.prev_sibling();
    }
    false
}

fn check_single_sibling(node: Node, source: &str) -> bool {
    is_comment_node(node.kind()) && node_text_contains(node, source, "SAFETY:")
}

fn node_text_contains(node: Node, source: &str, pat: &str) -> bool {
    node.utf8_text(source.as_bytes())
        .is_ok_and(|t| t.contains(pat))
}

fn is_comment_node(kind: &str) -> bool {
    kind == "line_comment" || kind == "block_comment"
}

fn check_lines_above(row: usize, source: &str) -> bool {
    if row == 0 {
        return false;
    }

    let lines: Vec<&str> = source.lines().collect();
    let start_check = row.saturating_sub(3);

    for i in (start_check..row).rev() {
        if let Some(line) = lines.get(i) {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if check_line_content(trimmed) {
                return true;
            }
            if !trimmed.starts_with("//") {
                return false;
            }
        }
    }
    false
}

fn check_line_content(trimmed: &str) -> bool {
    trimmed.starts_with("//") && trimmed.contains("SAFETY:")
}
