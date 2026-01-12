// src/analysis/v2/patterns/resource.rs
//! Resource patterns: R07 (missing flush)

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};
use super::get_capture_node;

#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut out = Vec::new();
    detect_r07(source, root, &mut out);
    out
}

/// R07: `BufWriter` created without `flush()` call
fn detect_r07(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r#"(call_expression
        function: (scoped_identifier path: (identifier) @type name: (identifier) @method)
        (#eq? @type "BufWriter") (#eq? @method "new")) @call"#;

    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else { return };
    let idx_call = query.capture_index_for_name("call");
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let Some(call) = get_capture_node(&m, idx_call) else { continue };
        let Some(fn_node) = find_containing_fn(call) else { continue };

        let fn_text = fn_node.utf8_text(source.as_bytes()).unwrap_or("");
        if fn_text.contains(".flush()") { continue }
        if fn_text.contains("-> BufWriter") || fn_text.contains("-> impl Write") { continue }
        if fn_text.contains("Ok(BufWriter") { continue }

        out.push(Violation::with_details(
            call.start_position().row + 1,
            "`BufWriter` without `flush()`".into(),
            "R07",
            ViolationDetails {
                function_name: None,
                analysis: vec!["Data may not be written on drop.".into()],
                suggestion: Some("Call `.flush()?` before scope ends.".into()),
            }
        ));
    }
}

fn find_containing_fn(node: Node) -> Option<Node> {
    let mut cur = node;
    loop {
        if cur.kind() == "function_item" { return Some(cur) }
        cur = cur.parent()?;
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
    fn r07_flag_missing_flush() {
        let code = "fn write_data() { let mut w = BufWriter::new(file); w.write_all(data); }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "R07"));
    }

    #[test]
    fn r07_skip_with_flush() {
        let code = "fn write_data() { let mut w = BufWriter::new(file); w.write_all(data); w.flush(); }";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "R07"));
    }

    #[test]
    fn r07_skip_returned_writer() {
        let code = "fn make() -> BufWriter<File> { Ok(BufWriter::new(f)) }";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "R07"));
    }
}