// src/analysis/v2/patterns/semantic.rs
//! Semantic patterns: M03, M04, M05

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor, QueryCapture};

#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut out = Vec::new();
    detect_m03(source, root, &mut out);
    detect_m04(source, root, &mut out);
    detect_m05(source, root, &mut out);
    out
}

fn cap_name<'a>(query: &'a Query, cap: &QueryCapture) -> &'a str {
    query.capture_names().get(cap.index as usize).map_or("", String::as_str)
}

/// M03: Getter with mutation - `get_*`/`is_*` that takes `&mut self`
fn detect_m03(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r#"(function_item
        name: (identifier) @name
        parameters: (parameters (self_parameter) @self)
        (#match? @name "^(get_|is_|has_)")) @fn"#;

    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else { return };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let fn_cap = m.captures.iter().find(|c| cap_name(&query, c) == "fn");
        let name_cap = m.captures.iter().find(|c| cap_name(&query, c) == "name");
        let self_cap = m.captures.iter().find(|c| cap_name(&query, c) == "self");

        let (Some(fn_cap), Some(name_cap), Some(self_cap)) = (fn_cap, name_cap, self_cap) else { continue };

        let self_text = self_cap.node.utf8_text(source.as_bytes()).unwrap_or("");
        if !self_text.contains("mut") { continue }

        let name = name_cap.node.utf8_text(source.as_bytes()).unwrap_or("");
        out.push(Violation::with_details(
            fn_cap.node.start_position().row + 1,
            format!("Getter `{name}` takes &mut self"),
            "M03",
            ViolationDetails {
                function_name: Some(name.to_string()),
                analysis: vec!["Getters should not mutate state.".into()],
                suggestion: Some("Rename or remove &mut.".into()),
            }
        ));
    }
}

/// M04: `is_*`/`has_*` that doesn't return bool
fn detect_m04(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r#"(function_item
        name: (identifier) @name
        return_type: (_) @ret
        (#match? @name "^(is_|has_|can_|should_)")) @fn"#;

    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else { return };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let fn_cap = m.captures.iter().find(|c| cap_name(&query, c) == "fn");
        let name_cap = m.captures.iter().find(|c| cap_name(&query, c) == "name");
        let ret_cap = m.captures.iter().find(|c| cap_name(&query, c) == "ret");

        let (Some(fn_cap), Some(name_cap), Some(ret_cap)) = (fn_cap, name_cap, ret_cap) else { continue };

        let ret = ret_cap.node.utf8_text(source.as_bytes()).unwrap_or("");
        if ret == "bool" { continue }

        let name = name_cap.node.utf8_text(source.as_bytes()).unwrap_or("");
        out.push(Violation::with_details(
            fn_cap.node.start_position().row + 1,
            format!("`{name}` returns `{ret}` not bool"),
            "M04",
            ViolationDetails {
                function_name: Some(name.to_string()),
                analysis: vec![format!("`is_*`/`has_*` should return bool.")],
                suggestion: Some("Rename or change return type.".into()),
            }
        ));
    }
}

/// M05: `calculate_*`/`compute_*` that takes `&mut self`
fn detect_m05(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r#"(function_item
        name: (identifier) @name
        parameters: (parameters (self_parameter) @self)
        (#match? @name "^(calculate_|compute_|count_|sum_)")) @fn"#;

    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else { return };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let fn_cap = m.captures.iter().find(|c| cap_name(&query, c) == "fn");
        let name_cap = m.captures.iter().find(|c| cap_name(&query, c) == "name");
        let self_cap = m.captures.iter().find(|c| cap_name(&query, c) == "self");

        let (Some(fn_cap), Some(name_cap), Some(self_cap)) = (fn_cap, name_cap, self_cap) else { continue };

        let self_text = self_cap.node.utf8_text(source.as_bytes()).unwrap_or("");
        if !self_text.contains("mut") { continue }

        let name = name_cap.node.utf8_text(source.as_bytes()).unwrap_or("");
        out.push(Violation::with_details(
            fn_cap.node.start_position().row + 1,
            format!("Calculator `{name}` takes &mut self"),
            "M05",
            ViolationDetails {
                function_name: Some(name.to_string()),
                analysis: vec!["Pure calculations should not mutate.".into()],
                suggestion: Some("Remove mutation or rename.".into()),
            }
        ));
    }
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
