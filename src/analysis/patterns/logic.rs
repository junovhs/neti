//! Logic boundary patterns: L02 (off-by-one risk), L03 (unchecked index).
//!
//! # L02 Design
//!
//! L02 flags `<=`/`>=` comparisons with `.len()` that risk an off-by-one
//! panic. Crucially, it must NOT flag canonical bounds guards:
//!
//!   ```ignore
//!   if idx >= v.len() { return None; } // safe guard — do NOT flag
//!   ```
//!
//! Only the dangerous direction (where idx could reach len as an array index)
//! should produce a violation:
//!
//!   ```ignore
//!   if i <= v.len() { process(v[i]); } // idx could equal len — DO flag
//!   ```
//!
//! # L03 Design
//!
//! L03 flags `[0]` and `.first().unwrap()` without a bounds proof. It must
//! NOT flag indexing that is provably safe due to iterator invariants:
//!
//!   ```ignore
//!   slice.chunks_exact(2).map(|a| u16::from_le_bytes([a[0], a[1]]));
//!   //                                                  ^^^^  safe
//!   ```
//!
//! L03 also must NOT flag constant indexing into fixed-size arrays:
//!
//!   ```ignore
//!   let seed = [0u8; 32]; seed[0] = 1;  // cannot panic
//!   self.s[0]  // where s: [u32; 4]     // cannot panic
//!   ```

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut out = Vec::new();
    detect_l02(source, root, &mut out);
    detect_l03(source, root, &mut out);
    out
}

// ── L02 ─────────────────────────────────────────────────────────────────────

/// L02: Boundary ambiguity — `<=` or `>=` with `.len()` in the dangerous direction.
fn detect_l02(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r"(binary_expression) @cmp";
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else {
        return;
    };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let Some(cmp) = m.captures.first().map(|c| c.node) else {
            continue;
        };

        let text = cmp.utf8_text(source.as_bytes()).unwrap_or("");
        if !text.contains(".len()") {
            continue;
        }
        if !text.contains("<=") && !text.contains(">=") {
            continue;
        }

        if is_safe_boundary(cmp, source) {
            continue;
        }

        out.push(Violation::with_details(
            cmp.start_position().row + 1,
            "Boundary uses `<=`/`>=` with `.len()` — possible off-by-one".into(),
            "L02",
            ViolationDetails {
                function_name: None,
                analysis: vec![
                    "Indices are 0..len-1. Comparing with `<= len` can reach `len`.".into(),
                ],
                suggestion: Some("Use `< len` for index upper bounds.".into()),
            },
        ));
    }
}

/// Returns `true` if this boundary comparison is provably safe (should not flag).
///
/// Safe cases:
/// - One side is a literal (threshold check, e.g. `v.len() >= 5`)
/// - `idx >= v.len()` — canonical early-return guard (idx is OUT of bounds)
/// - `v.len() <= idx` — same guard, reversed
///
/// Flaggable cases:
/// - `idx <= v.len()` — idx could equal len (off-by-one risk)
/// - `v.len() >= idx` — same risk, reversed
fn is_safe_boundary(node: Node, source: &str) -> bool {
    let left = node.child_by_field_name("left");
    let right = node.child_by_field_name("right");

    // Literal on either side → threshold check, always safe
    if is_literal(left) || is_literal(right) {
        return true;
    }

    let full_text = node.utf8_text(source.as_bytes()).unwrap_or("");
    let op = extract_op(full_text);

    let left_text = left
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("");
    let right_text = right
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("");

    if right_text.contains(".len()") {
        // Pattern: X OP v.len()
        // `>=`: x >= v.len() → guard ("x is out of bounds") — SAFE
        // `<=`: x <= v.len() → x could equal len — DANGEROUS
        if op == ">=" {
            return true;
        }
        return !is_index_variable(left_text);
    }

    if left_text.contains(".len()") {
        // Pattern: v.len() OP X
        // `<=`: v.len() <= x → same as x >= v.len() → guard — SAFE
        // `>=`: v.len() >= x → same as x <= v.len() → DANGEROUS
        if op == "<=" {
            return true;
        }
        return !is_index_variable(right_text);
    }

    // Ambiguous — err on the side of silence
    true
}

fn extract_op(full_text: &str) -> &str {
    // Order matters: check `<=` before `<`, `>=` before `>`
    if full_text.contains("<=") {
        "<="
    } else if full_text.contains(">=") {
        ">="
    } else {
        ""
    }
}

fn is_literal(node: Option<Node>) -> bool {
    node.is_some_and(|n| n.kind() == "integer_literal" || n.kind() == "float_literal")
}

fn is_index_variable(name: &str) -> bool {
    let n = name.trim();
    // Common short index names
    n == "i"
        || n == "j"
        || n == "k"
        || n == "n"
        || n == "idx"
        || n.contains("index")
        || n.contains("pos")
        || n.contains("ptr")
        || n.contains("offset")
        || n.contains("cursor")
}

// ── L03 ─────────────────────────────────────────────────────────────────────

/// L03: Unchecked `[0]` or `.first()/.last().unwrap()`.
fn detect_l03(source: &str, root: Node, out: &mut Vec<Violation>) {
    detect_index_zero(source, root, out);
    detect_first_last_unwrap(source, root, out);
}

fn detect_index_zero(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r"(index_expression) @idx";
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else {
        return;
    };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let Some(idx_node) = m.captures.first().map(|c| c.node) else {
            continue;
        };

        let text = idx_node.utf8_text(source.as_bytes()).unwrap_or("");
        if !text.ends_with("[0]") {
            continue;
        }

        // Explicit guard: caller already checks .is_empty() or .len()
        if has_explicit_guard(source, idx_node) {
            continue;
        }

        // Proof by iterator invariant: inside chunks_exact / array_chunks
        if has_chunks_exact_context(source, idx_node) {
            continue;
        }

        // Proof by fixed-size array: indexing a [T; N] with constant < N
        if is_fixed_size_array_access(source, idx_node, root) {
            continue;
        }

        out.push(Violation::with_details(
            idx_node.start_position().row + 1,
            "Index `[0]` without bounds check".into(),
            "L03",
            ViolationDetails {
                function_name: None,
                analysis: vec!["Panics on empty collection.".into()],
                suggestion: Some(
                    "Use `.first()` and handle `None`, or check `.is_empty()` first.".into(),
                ),
            },
        ));
    }
}

fn detect_first_last_unwrap(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r"(call_expression) @call";
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else {
        return;
    };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let Some(call) = m.captures.first().map(|c| c.node) else {
            continue;
        };

        let text = call.utf8_text(source.as_bytes()).unwrap_or("");
        if !text.contains(".first()") && !text.contains(".last()") {
            continue;
        }
        if !text.contains(".unwrap()") {
            continue;
        }
        if has_explicit_guard(source, call) {
            continue;
        }

        out.push(Violation::with_details(
            call.start_position().row + 1,
            "`.first()/.last().unwrap()` without guard".into(),
            "L03",
            ViolationDetails {
                function_name: None,
                analysis: vec!["Panics on empty collection.".into()],
                suggestion: Some("Use `?` or check `.is_empty()`.".into()),
            },
        ));
    }
}

/// Returns `true` if a `.len()` / `.is_empty()` guard is visible in the
/// ancestor chain — meaning the caller already proved non-emptiness.
fn has_explicit_guard(source: &str, node: Node) -> bool {
    let mut cur = node;
    for _ in 0..10 {
        let Some(p) = cur.parent() else { break };
        let text = p.utf8_text(source.as_bytes()).unwrap_or("");
        if text.contains(".len()") || text.contains(".is_empty()") {
            return true;
        }
        if p.kind() == "if_expression" && text.contains('!') && text.contains("is_empty") {
            return true;
        }
        cur = p;
    }
    false
}

/// Returns `true` if the node is inside a `chunks_exact(N)` or `array_chunks()`
/// iterator — meaning the chunk length is guaranteed by the iterator contract.
///
/// Example (safe — do NOT flag):
/// ```ignore
/// data.chunks_exact(2).map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]));
/// ```
fn has_chunks_exact_context(source: &str, node: Node) -> bool {
    let mut cur = node;
    for _ in 0..25 {
        let Some(p) = cur.parent() else { break };
        let text = p.utf8_text(source.as_bytes()).unwrap_or("");
        if text.contains("chunks_exact(") || text.contains("array_chunks(") {
            return true;
        }
        // Stop at source-file boundary
        if p.kind() == "source_file" {
            break;
        }
        cur = p;
    }
    false
}

// ── Fixed-size array proof ──────────────────────────────────────────────────

/// Returns `true` if this index expression accesses a fixed-size array with a
/// constant index that is provably within bounds.
///
/// Covers:
/// 1. Local `let arr = [0u8; 32]; arr[0]` — repeat expression `[expr; N]`
/// 2. Local `let arr = [1, 2, 3]; arr[0]` — array literal
/// 3. Local `let arr: [u8; 4] = ...; arr[0]` — type annotation
/// 4. Struct field `self.s[0]` where `s: [u32; 4]`
/// 5. Function param `fn f(buf: [u8; 4]) { buf[0] }`
fn is_fixed_size_array_access(source: &str, idx_node: Node, root: Node) -> bool {
    let text = idx_node.utf8_text(source.as_bytes()).unwrap_or("");

    let Some(index_val) = extract_constant_index(text) else {
        return false;
    };

    let receiver = extract_receiver(text);

    // Strategy 1: local variable declaration
    if let Some(size) = find_local_array_size(source, idx_node, receiver) {
        return index_val < size;
    }

    // Strategy 2: struct field via self.field
    if let Some(field_name) = receiver.strip_prefix("self.") {
        // Only handle simple field names (no nested dots)
        if !field_name.contains('.') {
            if let Some(size) = find_struct_field_array_size(source, idx_node, root, field_name) {
                return index_val < size;
            }
        }
    }

    // Strategy 3: function parameter with array type annotation
    if let Some(size) = find_param_array_size(source, idx_node, receiver) {
        return index_val < size;
    }

    false
}

/// Extracts a constant integer index from text like `foo[0]`, `self.s[3]`.
fn extract_constant_index(text: &str) -> Option<usize> {
    let bracket_start = text.rfind('[')?;
    let bracket_end = text.rfind(']')?;
    if bracket_end <= bracket_start {
        return None;
    }
    let inner = text[bracket_start + 1..bracket_end].trim();
    inner.parse::<usize>().ok()
}

/// Extracts the receiver from an index expression: `self.s[0]` → `self.s`
fn extract_receiver(text: &str) -> &str {
    text.rfind('[').map_or(text, |pos| text[..pos].trim())
}

/// Walks up from the index node to find a `let` declaration for the receiver
/// variable and extracts array size from the initializer or type annotation.
fn find_local_array_size(source: &str, node: Node, receiver: &str) -> Option<usize> {
    if receiver.contains('.') {
        return None;
    }

    let mut cur = node;
    for _ in 0..30 {
        let Some(p) = cur.parent() else { break };

        if matches!(p.kind(), "block" | "function_item" | "source_file") {
            let mut child_cursor = p.walk();
            for child in p.children(&mut child_cursor) {
                if child.kind() != "let_declaration" {
                    continue;
                }
                if child.start_byte() >= node.start_byte() {
                    continue;
                }
                let decl_text = child.utf8_text(source.as_bytes()).unwrap_or("");
                if !decl_matches_variable(decl_text, receiver) {
                    continue;
                }
                if let Some(size) = extract_array_size_from_decl(decl_text) {
                    return Some(size);
                }
            }
            if matches!(p.kind(), "function_item" | "source_file") {
                break;
            }
        }
        cur = p;
    }
    None
}

/// Checks if a `let` declaration text declares the given variable name.
fn decl_matches_variable(decl_text: &str, var_name: &str) -> bool {
    let after_let = decl_text.strip_prefix("let").unwrap_or(decl_text).trim();
    let after_mut = after_let.strip_prefix("mut").unwrap_or(after_let).trim();
    after_mut.starts_with(var_name) && after_mut[var_name.len()..].starts_with([' ', ':', '=', ';'])
}

/// Extracts array size from a let declaration initializer or type annotation.
fn extract_array_size_from_decl(decl_text: &str) -> Option<usize> {
    // Pattern 1: repeat expression `[expr; N]` in initializer
    if let Some(size) = extract_repeat_array_size(decl_text) {
        return Some(size);
    }
    // Pattern 2: type annotation `[T; N]`
    if let Some(size) = extract_type_array_size(decl_text) {
        return Some(size);
    }
    // Pattern 3: array literal `[a, b, c]`
    extract_literal_array_size(decl_text)
}

/// Extracts N from `[expr; N]` repeat expressions.
fn extract_repeat_array_size(text: &str) -> Option<usize> {
    // Find the initializer part (after `=`)
    let eq_pos = text.find('=')?;
    let after_eq = text[eq_pos + 1..].trim();

    if !after_eq.starts_with('[') {
        return None;
    }
    let bracket_end = after_eq.find(']')?;
    let inner = &after_eq[1..bracket_end];

    let semi_pos = inner.rfind(';')?;
    let size_str = inner[semi_pos + 1..].trim();
    parse_size_literal(size_str)
}

/// Extracts N from `: [T; N]` type annotations.
fn extract_type_array_size(text: &str) -> Option<usize> {
    let colon_pos = text.find(':')?;
    let after_colon = &text[colon_pos + 1..];

    // Find `[` that starts the array type (skip past `=` if any)
    let eq_pos = after_colon.find('=').unwrap_or(after_colon.len());
    let type_region = &after_colon[..eq_pos];

    let bracket_start = type_region.find('[')?;
    let bracket_end = type_region.find(']')?;
    if bracket_end <= bracket_start {
        return None;
    }
    let inner = &type_region[bracket_start + 1..bracket_end];
    let semi_pos = inner.rfind(';')?;
    let size_str = inner[semi_pos + 1..].trim();
    parse_size_literal(size_str)
}

/// Counts elements in an array literal `[a, b, c]`.
fn extract_literal_array_size(text: &str) -> Option<usize> {
    let eq_pos = text.find('=')?;
    let after_eq = text[eq_pos + 1..].trim();

    if !after_eq.starts_with('[') {
        return None;
    }
    let bracket_end = after_eq.find(']')?;
    let inner = &after_eq[1..bracket_end];

    // If it contains a semicolon, it's a repeat expression, not a literal
    if inner.contains(';') {
        return None;
    }

    let trimmed = inner.trim();
    if trimmed.is_empty() {
        return Some(0);
    }

    Some(trimmed.split(',').count())
}

/// Parses a size literal that may have a type suffix: `32`, `32usize`, etc.
fn parse_size_literal(s: &str) -> Option<usize> {
    let cleaned = s
        .trim()
        .trim_end_matches("usize")
        .trim_end_matches("u32")
        .trim_end_matches("u64")
        .trim_end_matches("i32")
        .trim_end_matches("i64")
        .trim();
    cleaned.parse::<usize>().ok()
}

/// Searches for a struct field's array type size by finding the struct
/// definition in the same file.
fn find_struct_field_array_size(
    source: &str,
    node: Node,
    root: Node,
    field_name: &str,
) -> Option<usize> {
    // Walk up to find the impl block and get the type name
    let type_name = find_enclosing_impl_type(source, node)?;

    // Scan top-level items for the struct definition
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() != "struct_item" {
            continue;
        }
        let struct_text = child.utf8_text(source.as_bytes()).unwrap_or("");

        if !struct_text.contains(&format!("struct {type_name}"))
            && !struct_text.contains(&format!("struct {type_name}<"))
        {
            continue;
        }

        // Search each line for the field declaration
        for line in struct_text.lines() {
            if let Some(size) = extract_field_array_size(line, field_name) {
                return Some(size);
            }
        }
    }
    None
}

/// Walks up from a node to find the enclosing `impl` block's type name.
fn find_enclosing_impl_type<'a>(source: &'a str, node: Node) -> Option<&'a str> {
    let mut cur = node;
    for _ in 0..30 {
        let Some(p) = cur.parent() else { break };
        if p.kind() == "impl_item" {
            let impl_text = p.utf8_text(source.as_bytes()).unwrap_or("");
            return extract_impl_type_name(impl_text);
        }
        if p.kind() == "source_file" {
            break;
        }
        cur = p;
    }
    None
}

/// Extracts the type name from `impl TypeName { ... }` or
/// `impl Trait for TypeName { ... }`.
fn extract_impl_type_name(impl_text: &str) -> Option<&str> {
    let first_line = impl_text.lines().next()?;
    let after_impl = first_line.strip_prefix("impl")?.trim();

    // Skip generic parameters `<T, U>`
    let after_generics = if after_impl.starts_with('<') {
        let mut depth = 0;
        let mut end = 0;
        for (i, c) in after_impl.char_indices() {
            match c {
                '<' => depth += 1,
                '>' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        after_impl[end..].trim()
    } else {
        after_impl
    };

    // Handle `Trait for TypeName`
    let type_part = if let Some(pos) = after_generics.find(" for ") {
        after_generics[pos + 5..].trim()
    } else {
        after_generics
    };

    let name = type_part
        .split(|c: char| c == '<' || c == '{' || c.is_whitespace())
        .next()?;

    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// Extracts array size from a struct field line like `s: [u32; 4],` or
/// `pub s: [u32; 4],`.
fn extract_field_array_size(line: &str, field_name: &str) -> Option<usize> {
    let trimmed = line.trim();
    let after_pub = trimmed.strip_prefix("pub ").unwrap_or(trimmed).trim();

    if !after_pub.starts_with(field_name) {
        return None;
    }
    let after_name = &after_pub[field_name.len()..];
    if !after_name.starts_with(':') {
        return None;
    }
    let type_str = after_name[1..].trim();

    if !type_str.starts_with('[') {
        return None;
    }
    let bracket_end = type_str.find(']')?;
    let inner = &type_str[1..bracket_end];
    let semi_pos = inner.rfind(';')?;
    let size_str = inner[semi_pos + 1..].trim().trim_end_matches(',');
    size_str.parse::<usize>().ok()
}

/// Checks if the receiver is a function parameter with array type annotation.
fn find_param_array_size(source: &str, node: Node, receiver: &str) -> Option<usize> {
    if receiver.contains('.') {
        return None;
    }

    let mut cur = node;
    for _ in 0..20 {
        let Some(p) = cur.parent() else { break };

        if matches!(p.kind(), "function_item" | "closure_expression") {
            let fn_text = p.utf8_text(source.as_bytes()).unwrap_or("");
            // Look for `receiver: [T; N]` in the function signature
            let pattern = format!("{receiver}:");
            if let Some(pos) = fn_text.find(&pattern) {
                let after = fn_text[pos + pattern.len()..].trim();
                if after.starts_with('[') {
                    if let Some(bracket_end) = after.find(']') {
                        let inner = &after[1..bracket_end];
                        if let Some(semi_pos) = inner.rfind(';') {
                            let size_str = inner[semi_pos + 1..].trim();
                            if let Ok(size) = size_str.parse::<usize>() {
                                return Some(size);
                            }
                        }
                    }
                }
            }
            if p.kind() == "function_item" {
                break;
            }
        }
        cur = p;
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

    // ── L02 ──────────────────────────────────────────────────────────────

    #[test]
    fn l02_flag_lte_len() {
        let code = "fn f(v: &[i32], i: usize) -> bool { i <= v.len() }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "L02"));
    }

    #[test]
    fn l02_flag_len_gte_idx() {
        let code = "fn f(v: &[i32], i: usize) -> bool { v.len() >= i }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "L02"));
    }

    #[test]
    fn l02_skip_threshold() {
        let code = "fn f(v: &[i32]) -> bool { v.len() >= 5 }";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "L02"));
    }

    #[test]
    fn l02_skip_max_var() {
        let code = "fn f(v: &[i32], max: usize) -> bool { v.len() <= max }";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "L02"));
    }

    #[test]
    fn l02_skip_guard_idx_gte_len() {
        let code = "fn f(v: &[i32], idx: usize) -> Option<i32> { if idx >= v.len() { return None; } Some(v[0]) }";
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "L02"),
            "idx >= v.len() is a canonical guard and must not be flagged"
        );
    }

    #[test]
    fn l02_skip_guard_len_lte_idx() {
        let code = "fn f(v: &[i32], idx: usize) -> Option<i32> { if v.len() <= idx { return None; } Some(v[0]) }";
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "L02"),
            "v.len() <= idx is a guard and must not be flagged"
        );
    }

    // ── L03 ──────────────────────────────────────────────────────────────

    #[test]
    fn l03_flag_index_zero() {
        let code = "fn f(v: &[i32]) -> i32 { v[0] }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "L03"));
    }

    #[test]
    fn l03_skip_with_empty_check() {
        let code = "fn f(v: &[i32]) -> i32 { if !v.is_empty() { v[0] } else { 0 } }";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "L03"));
    }

    #[test]
    fn l03_flag_first_unwrap() {
        let code = "fn f(v: &[i32]) -> i32 { *v.first().unwrap() }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "L03"));
    }

    #[test]
    fn l03_skip_chunks_exact_index() {
        let code = r"
            fn f(data: &[u8]) -> Vec<u16> {
                data.chunks_exact(2)
                    .map(|a| u16::from_le_bytes([a[0], a[1]]))
                    .collect()
            }
        ";
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "L03"),
            "chunks_exact indexing is provably safe and must not be flagged"
        );
    }

    // ── L03 fixed-size array ─────────────────────────────────────────────

    #[test]
    fn l03_skip_fixed_array_repeat() {
        let code = r"
            fn f() {
                let mut seed = [0u8; 32];
                seed[0] = 1;
            }
        ";
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "L03"),
            "seed[0] on [0u8; 32] is provably safe"
        );
    }

    #[test]
    fn l03_skip_fixed_array_literal() {
        let code = r"
            fn f() -> i32 {
                let arr = [1, 2, 3];
                arr[0]
            }
        ";
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "L03"),
            "arr[0] on [1, 2, 3] is provably safe"
        );
    }

    #[test]
    fn l03_skip_struct_field_array() {
        let code = r"
            struct Rng {
                s: [u32; 4],
            }
            impl Rng {
                fn next(&mut self) -> u32 {
                    let res = self.s[0].wrapping_add(self.s[3]);
                    res
                }
            }
        ";
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "L03"),
            "self.s[0] on [u32; 4] is provably safe"
        );
    }

    #[test]
    fn l03_skip_typed_param_array() {
        let code = r"
            fn process(buf: [u8; 4]) -> u8 {
                buf[0]
            }
        ";
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "L03"),
            "buf[0] on [u8; 4] parameter is provably safe"
        );
    }

    #[test]
    fn l03_still_flags_vec_index() {
        let code = r"
            fn f(v: Vec<i32>) -> i32 {
                v[0]
            }
        ";
        assert!(
            parse_and_detect(code).iter().any(|v| v.law == "L03"),
            "v[0] on Vec<i32> should still be flagged"
        );
    }

    #[test]
    fn l03_still_flags_slice_index() {
        let code = "fn f(v: &[i32]) -> i32 { v[0] }";
        assert!(
            parse_and_detect(code).iter().any(|v| v.law == "L03"),
            "v[0] on &[i32] should still be flagged"
        );
    }
}
