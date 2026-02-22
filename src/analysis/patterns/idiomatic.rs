//! Idiomatic patterns: I01, I02.
//!
//! # I01 Design
//!
//! I01 detects manual `From` implementations that could potentially use
//! `derive_more::From`. However, this is a STYLE suggestion, not a
//! correctness issue. Many foundational crates intentionally avoid proc-macro
//! dependencies. I01 is reported as a suggestion, not an error.
//!
//! # I02 Design
//!
//! I02 detects match arms with duplicate bodies that could be combined using
//! `A | B => body`. However, this is ONLY valid when the arm bindings have
//! compatible types. Enum variants wrapping different inner types (e.g.,
//! `Foo::U32(v)` vs `Foo::U64(v)`) produce textually identical bodies
//! that CANNOT be fused because `v` has a different type in each arm.
//!
//! I02 must detect this case and suppress the suggestion.

use crate::types::{Violation, ViolationDetails};
use std::collections::HashMap;
use tree_sitter::{Node, Query, QueryCursor};

#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut out = Vec::new();
    detect_i01(source, root, &mut out);
    detect_i02(source, root, &mut out);
    out
}

// ── I01 ─────────────────────────────────────────────────────────────────────

/// Detects manual `From` implementations by scanning for `impl_item` nodes
/// whose text matches the `impl From<...> for ...` pattern.
///
/// We avoid relying on tree-sitter field names (`type:`, `trait:`) because
/// they vary across grammar versions. Instead we do a text-based check on
/// the impl header, which is simple and robust.
///
/// Reported as a style suggestion — many crates intentionally avoid
/// proc-macro dependencies and manual From impls are perfectly valid.
fn detect_i01(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r"(impl_item) @impl";
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else {
        return;
    };

    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, root, source.as_bytes()) {
        let Some(impl_node) = m.captures.first().map(|c| c.node) else {
            continue;
        };

        let impl_text = impl_node.utf8_text(source.as_bytes()).unwrap_or("");

        // Extract the first line to check the impl header
        let header = impl_text.lines().next().unwrap_or("");

        // Must be `impl From<...> for ...`
        if !is_from_impl(header) {
            continue;
        }

        // Skip Error → Self From impls (idiomatic for error types)
        if is_error_from_impl(impl_text) {
            continue;
        }

        out.push(Violation::with_details(
            impl_node.start_position().row + 1,
            "Manual `From` impl — consider derive_more if already using proc macros".into(),
            "I01",
            ViolationDetails {
                function_name: None,
                analysis: vec![
                    "This `From` impl could be generated with `#[derive(From)]`.".into(),
                    "Note: many crates intentionally avoid proc-macro dependencies.".into(),
                ],
                suggestion: Some(
                    "Use derive_more::From if your crate already uses proc macros. Otherwise this is fine as-is.".into(),
                ),
            },
        ));
    }
}

/// Checks if an impl header line is a `From` impl.
fn is_from_impl(header: &str) -> bool {
    let trimmed = header.trim();
    let after_impl = match trimmed.strip_prefix("impl") {
        Some(rest) => rest.trim_start(),
        None => return false,
    };
    after_impl.starts_with("From<") || after_impl.starts_with("From ")
}

fn is_error_from_impl(impl_text: &str) -> bool {
    let lower = impl_text.to_lowercase();
    lower.contains("error") || lower.contains("err")
}

// ── I02 ─────────────────────────────────────────────────────────────────────

fn detect_i02(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r"(match_expression body: (match_block) @block) @match";
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else {
        return;
    };
    let idx_match = query.capture_index_for_name("match");
    let idx_block = query.capture_index_for_name("block");

    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, root, source.as_bytes()) {
        let Some(match_node) =
            idx_match.and_then(|i| m.captures.iter().find(|c| c.index == i).map(|c| c.node))
        else {
            continue;
        };
        let Some(block) =
            idx_block.and_then(|i| m.captures.iter().find(|c| c.index == i).map(|c| c.node))
        else {
            continue;
        };

        check_duplicate_arms(source, match_node, block, out);
    }
}

fn check_duplicate_arms(source: &str, match_node: Node, block: Node, out: &mut Vec<Violation>) {
    let mut arm_bodies: HashMap<String, usize> = HashMap::new();
    let mut arm_patterns: HashMap<String, Vec<String>> = HashMap::new();

    let mut cursor = block.walk();
    for child in block.children(&mut cursor) {
        if child.kind() != "match_arm" {
            continue;
        }

        let pattern_node = child.child_by_field_name("pattern");
        let body = arm_body_text(source, child);

        let pattern_text = pattern_node
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .unwrap_or("")
            .to_string();

        if let Some(body_text) = body {
            let trimmed = body_text.trim().to_string();
            if trimmed.is_empty() || trimmed == "_" {
                continue;
            }
            *arm_bodies.entry(trimmed.clone()).or_insert(0) += 1;
            arm_patterns.entry(trimmed).or_default().push(pattern_text);
        }
    }

    for (body, count) in &arm_bodies {
        if *count < 2 {
            continue;
        }

        let patterns = arm_patterns.get(body).unwrap_or(&Vec::new()).clone();

        if patterns_have_incompatible_types(&patterns) {
            continue;
        }

        let truncated = if body.len() > 30 {
            format!("{}...", &body[..30])
        } else {
            body.clone()
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

/// Returns `true` if the set of patterns destructure enum variants that likely
/// have incompatible inner types, making arm fusion impossible.
///
/// Handles two forms:
/// 1. Simple: `Enum::VariantA(v)` vs `Enum::VariantB(v)`
/// 2. Tuple:  `(Enum::A(x), Enum::A(y))` vs `(Enum::B(x), Enum::B(y))`
fn patterns_have_incompatible_types(patterns: &[String]) -> bool {
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

/// Extracts enum variant names from a pattern string.
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

/// Extracts a single variant name from a pattern like `Enum::Variant(x)`.
fn extract_single_variant_name(pattern: &str) -> Option<&str> {
    let paren_pos = pattern.find('(')?;
    let path = pattern[..paren_pos].trim();

    if path.is_empty() {
        return None;
    }

    Some(path.rsplit("::").next().unwrap_or(path))
}

/// Splits a string on commas at the top level only (not inside parentheses).
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

/// Extracts the body text from a match arm.
fn arm_body_text(source: &str, arm: Node) -> Option<String> {
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
            return Some(trimmed.to_string());
        }
    }

    None
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_and_detect(code: &str) -> Vec<Violation> {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_rust::language()).unwrap();
        let tree = parser.parse(code, None).unwrap();
        detect(code, tree.root_node())
    }

    #[test]
    fn i01_flag_simple_from() {
        let code = r#"
            impl From<String> for MyType {
                fn from(s: String) -> Self { MyType(s) }
            }
        "#;
        assert!(parse_and_detect(code).iter().any(|v| v.law == "I01"));
    }

    #[test]
    fn i01_skip_error_from() {
        let code = r#"
            impl From<IoError> for MyError {
                fn from(e: IoError) -> Self { MyError::Io(e) }
            }
        "#;
        assert!(parse_and_detect(code).iter().all(|v| v.law != "I01"));
    }

    #[test]
    fn i01_message_is_suggestion_not_error() {
        let code = r#"
            impl From<String> for MyType {
                fn from(s: String) -> Self { MyType(s) }
            }
        "#;
        let violations = parse_and_detect(code);
        let i01 = violations.iter().find(|v| v.law == "I01").unwrap();
        assert!(
            i01.message.contains("consider"),
            "I01 message should be suggestive, not imperative"
        );
    }

    #[test]
    fn i02_flag_duplicate_arms() {
        let code = r#"
            fn f(x: Option<i32>) -> &str {
                match x {
                    Some(_) => "yes",
                    None => "yes",
                }
            }
        "#;
        assert!(parse_and_detect(code).iter().any(|v| v.law == "I02"));
    }

    #[test]
    fn i02_skip_unique_arms() {
        let code = r#"
            fn f(x: Option<i32>) -> &str {
                match x {
                    Some(_) => "yes",
                    None => "no",
                }
            }
        "#;
        assert!(parse_and_detect(code).iter().all(|v| v.law != "I02"));
    }

    #[test]
    fn i02_skip_different_variant_types() {
        let code = r#"
            enum IndexVec {
                U32(Vec<u32>),
                U64(Vec<u64>),
            }
            impl IndexVec {
                fn len(&self) -> usize {
                    match self {
                        IndexVec::U32(v) => v.len(),
                        IndexVec::U64(v) => v.len(),
                    }
                }
            }
        "#;
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "I02"),
            "Different variant types must not be flagged as fuseable"
        );
    }

    #[test]
    fn i02_skip_tuple_match_different_variants() {
        let code = r#"
            enum Idx { U32(Vec<u32>), U64(Vec<u64>) }
            impl PartialEq for Idx {
                fn eq(&self, other: &Self) -> bool {
                    use Idx::*;
                    match (self, other) {
                        (U32(v1), U32(v2)) => v1 == v2,
                        (U64(v1), U64(v2)) => v1 == v2,
                        _ => false,
                    }
                }
            }
        "#;
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "I02"),
            "Tuple match with different variant types must not be flagged"
        );
    }

    #[test]
    fn i02_still_flags_same_variant_duplicates() {
        let code = r#"
            fn f(x: i32) -> &str {
                match x {
                    1 => "same",
                    2 => "same",
                    _ => "other",
                }
            }
        "#;
        assert!(
            parse_and_detect(code).iter().any(|v| v.law == "I02"),
            "Literal patterns with same body should still be flagged"
        );
    }
}
