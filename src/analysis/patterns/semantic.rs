// src/analysis/v2/patterns/semantic.rs
//! Semantic patterns: M03, M04, M05

use super::get_capture_node;
use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut out = Vec::new();
    detect_m03(source, root, &mut out);
    detect_m04(source, root, &mut out);
    detect_m05(source, root, &mut out);
    out
}

/// M03: Getter with mutation - `get_*`/`is_*` that takes `&mut self`
fn detect_m03(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r#"(function_item
        name: (identifier) @name
        parameters: (parameters (self_parameter) @self)
        (#match? @name "^(get_|is_|has_)")) @fn"#;

    let Ok(query) = Query::new(&tree_sitter_rust::LANGUAGE.into(), q) else {
        return;
    };
    let idx_fn = query.capture_index_for_name("fn");
    let idx_name = query.capture_index_for_name("name");
    let idx_self = query.capture_index_for_name("self");

    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let fn_cap = get_capture_node(&m, idx_fn);
        let name_cap = get_capture_node(&m, idx_name);
        let self_cap = get_capture_node(&m, idx_self);

        let (Some(fn_cap), Some(name_cap), Some(self_cap)) = (fn_cap, name_cap, self_cap) else {
            continue;
        };

        let self_text = self_cap.utf8_text(source.as_bytes()).unwrap_or("");
        if !self_text.contains("mut") {
            continue;
        }

        let name = name_cap.utf8_text(source.as_bytes()).unwrap_or("");
        emit_violation(
            out,
            fn_cap,
            name,
            "M03",
            "Getter",
            "Getters should not mutate state.",
            "Rename or remove &mut.",
        );
    }
}

/// M04: `is_*`/`has_*` that doesn't return bool
fn detect_m04(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r#"(function_item
        name: (identifier) @name
        return_type: (_) @ret
        (#match? @name "^(is_|has_|can_|should_)")) @fn"#;

    let Ok(query) = Query::new(&tree_sitter_rust::LANGUAGE.into(), q) else {
        return;
    };
    let idx_fn = query.capture_index_for_name("fn");
    let idx_name = query.capture_index_for_name("name");
    let idx_ret = query.capture_index_for_name("ret");

    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        check_m04_match(source, &m, idx_fn, idx_name, idx_ret, out);
    }
}

fn check_m04_match(
    source: &str,
    m: &tree_sitter::QueryMatch,
    idx_fn: Option<u32>,
    idx_name: Option<u32>,
    idx_ret: Option<u32>,
    out: &mut Vec<Violation>,
) {
    let fn_cap = get_capture_node(m, idx_fn);
    let name_cap = get_capture_node(m, idx_name);
    let ret_cap = get_capture_node(m, idx_ret);

    let (Some(fn_cap), Some(name_cap), Some(ret_cap)) = (fn_cap, name_cap, ret_cap) else {
        return;
    };

    let ret = ret_cap.utf8_text(source.as_bytes()).unwrap_or("");
    if ret == "bool" {
        return;
    }

    let name = name_cap.utf8_text(source.as_bytes()).unwrap_or("");
    let owned_name = name.to_string();
    out.push(Violation::with_details(
        fn_cap.start_position().row + 1,
        format!("`{name}` returns `{ret}` not bool"),
        "M04",
        ViolationDetails {
            function_name: Some(owned_name),
            analysis: vec!["`is_*`/`has_*` should return bool.".to_string()],
            suggestion: Some("Rename or change return type.".into()),
        },
    ));
}

/// M05: `calculate_*`/`compute_*` that takes `&mut self`
fn detect_m05(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r#"(function_item
        name: (identifier) @name
        parameters: (parameters (self_parameter) @self)
        (#match? @name "^(calculate_|compute_|count_|sum_)")) @fn"#;

    let Ok(query) = Query::new(&tree_sitter_rust::LANGUAGE.into(), q) else {
        return;
    };
    let idx_fn = query.capture_index_for_name("fn");
    let idx_name = query.capture_index_for_name("name");
    let idx_self = query.capture_index_for_name("self");

    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let fn_cap = get_capture_node(&m, idx_fn);
        let name_cap = get_capture_node(&m, idx_name);
        let self_cap = get_capture_node(&m, idx_self);

        let (Some(fn_cap), Some(name_cap), Some(self_cap)) = (fn_cap, name_cap, self_cap) else {
            continue;
        };

        let self_text = self_cap.utf8_text(source.as_bytes()).unwrap_or("");
        if !self_text.contains("mut") {
            continue;
        }

        let name = name_cap.utf8_text(source.as_bytes()).unwrap_or("");
        emit_violation(
            out,
            fn_cap,
            name,
            "M05",
            "Calculator",
            "Pure calculations should not mutate.",
            "Remove mutation or rename.",
        );
    }
}

fn emit_violation(
    out: &mut Vec<Violation>,
    fn_cap: Node,
    name: &str,
    code: &'static str,
    label: &str,
    analysis: &str,
    suggestion: &str,
) {
    let owned_name = name.to_string();
    out.push(Violation::with_details(
        fn_cap.start_position().row + 1,
        format!("{label} `{name}` takes &mut self"),
        code,
        ViolationDetails {
            function_name: Some(owned_name),
            analysis: vec![analysis.to_string()],
            suggestion: Some(suggestion.into()),
        },
    ));
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_and_detect(code: &str) -> Vec<Violation> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(code, None).unwrap();
        detect(code, tree.root_node())
    }

    #[test]
    fn m03_flag_getter_with_mut() {
        let code = "impl X { fn get_count(&mut self) -> usize { self.count } }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "M03"));
    }

    #[test]
    fn m03_skip_getter_without_mut() {
        let code = "impl X { fn get_count(&self) -> usize { self.count } }";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "M03"));
    }

    #[test]
    fn m04_flag_is_returning_string() {
        let code = "fn is_valid(x: i32) -> String { x.to_string() }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "M04"));
    }

    #[test]
    fn m04_skip_is_returning_bool() {
        let code = "fn is_valid(x: i32) -> bool { x > 0 }";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "M04"));
    }

    #[test]
    fn m05_flag_calculate_with_mut() {
        let code = "impl X { fn calculate_avg(&mut self) -> f64 { 0.0 } }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "M05"));
    }
}
