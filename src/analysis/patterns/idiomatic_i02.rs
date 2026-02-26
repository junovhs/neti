// src/analysis/patterns/idiomatic_i02.rs
//! I02: Duplicate match arm bodies that could be combined using `A | B => body`.
//!
//! ONLY valid when arm bindings have compatible types. Enum variants wrapping
//! different inner types produce textually identical bodies that CANNOT be fused.
//! I02 must detect this and suppress the suggestion.

use crate::types::{Violation, ViolationDetails};
use std::collections::HashMap;
use tree_sitter::{Node, Query, QueryCursor};

#[cfg(test)]
#[path = "idiomatic_i02_test.rs"]
mod tests;

pub(super) fn detect_i02(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r"(match_expression body: (match_block) @block) @match";
    let Ok(query) = Query::new(&tree_sitter_rust::LANGUAGE.into(), q) else {
        return;
    };
    let idx_match = query.capture_index_for_name("match");
    let idx_block = query.capture_index_for_name("block");

    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, root, source.as_bytes()) {
        let mut match_node = None;
        let mut block = None;

        for cap in m.captures {
            if Some(cap.index) == idx_match {
                match_node = Some(cap.node);
            }
            if Some(cap.index) == idx_block {
                block = Some(cap.node);
            }
        }

        let (Some(match_node), Some(block)) = (match_node, block) else {
            continue;
        };

        check_duplicate_arms(source, match_node, block, out);
    }
}

fn check_duplicate_arms(source: &str, match_node: Node, block: Node, out: &mut Vec<Violation>) {
    let mut arm_bodies: HashMap<&str, usize> = HashMap::new();
    let mut arm_patterns: HashMap<&str, Vec<&str>> = HashMap::new();

    let mut cursor = block.walk();
    for child in block.children(&mut cursor) {
        if child.kind() != "match_arm" {
            continue;
        }

        let pattern_node = child.child_by_field_name("pattern");
        let body = arm_body_text(source, child);

        let pattern_text = pattern_node
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .unwrap_or("");

        if let Some(body_text) = body {
            let trimmed = body_text.trim();
            if trimmed.is_empty() || trimmed == "_" {
                continue;
            }
            *arm_bodies.entry(trimmed).or_insert(0) += 1;
            arm_patterns.entry(trimmed).or_default().push(pattern_text);
        }
    }

    let empty_patterns = Vec::new();
    for (body, count) in &arm_bodies {
        if *count < 2 {
            continue;
        }

        let patterns = arm_patterns.get(body).unwrap_or(&empty_patterns);

        if patterns_have_incompatible_types(patterns) {
            continue;
        }

        let truncated = if body.len() > 30 {
            format!("{}...", &body[..30])
        } else {
            body.to_string()
        };

        out.push(Violation::with_details(
            match_node.start_position().row + 1,
            "Duplicate match arm bodies".into(),
            "I02",
            ViolationDetails {
                function_name: None,
                analysis: vec![format!("Duplicate: `{truncated}`")],
                suggestion: Some("Combine: `A | B => body`".into()),
            },
        ));
    }
}

fn patterns_have_incompatible_types(patterns: &[&str]) -> bool {
    if patterns.len() < 2 {
        return false;
    }

    let mut per_arm_variants: Vec<Vec<&str>> = Vec::new();

    for pat in patterns {
        let trimmed = pat.trim();

        if !trimmed.contains('(') {
            return false;
        }

        let variants = extract_variant_names(trimmed);
        if variants.is_empty() {
            return false;
        }

        per_arm_variants.push(variants);
    }

    if per_arm_variants.len() < 2 {
        return false;
    }

    let expected_count = per_arm_variants[0].len();
    if !per_arm_variants.iter().all(|v| v.len() == expected_count) {
        return false;
    }

    for slot in 0..expected_count {
        let names_in_slot: Vec<&str> = per_arm_variants.iter().map(|v| v[slot]).collect();

        let unique_count = {
            let mut sorted = names_in_slot.clone();
            sorted.sort();
            sorted.dedup();
            sorted.len()
        };

        if unique_count == names_in_slot.len() && unique_count >= 2 {
            return true;
        }
    }

    false
}

fn extract_variant_names(pattern: &str) -> Vec<&str> {
    let trimmed = pattern.trim();

    if trimmed.starts_with('(') {
        let inner = trimmed
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or(trimmed);

        let parts = split_top_level_commas(inner);
        let mut names = Vec::new();
        for part in &parts {
            let part = part.trim();
            if let Some(name) = extract_single_variant_name(part) {
                names.push(name);
            }
        }
        return names;
    }

    if let Some(name) = extract_single_variant_name(trimmed) {
        return vec![name];
    }

    Vec::new()
}

fn extract_single_variant_name(pattern: &str) -> Option<&str> {
    let paren_pos = pattern.find('(')?;
    let path = pattern[..paren_pos].trim();

    if path.is_empty() {
        return None;
    }

    Some(path.rsplit("::").next().unwrap_or(path))
}

fn split_top_level_commas(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, c) in s.char_indices() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&s[start..]);
    parts
}

fn arm_body_text<'a>(source: &'a str, arm: Node) -> Option<&'a str> {
    let child_count = arm.child_count();
    if child_count == 0 {
        return None;
    }

    let mut found_arrow = false;
    let mut cursor = arm.walk();
    for child in arm.children(&mut cursor) {
        if child.kind() == "=>" {
            found_arrow = true;
            continue;
        }
        if found_arrow {
            let text = child.utf8_text(source.as_bytes()).ok()?;
            let trimmed = text.trim().trim_end_matches(',').trim();
            return Some(trimmed);
        }
    }

    None
}
