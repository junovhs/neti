// src/analysis/checks/syntax.rs
use tree_sitter::Node;
use crate::types::Violation;
use super::CheckContext;

/// Checks for syntax errors or missing nodes in the AST.
pub fn check_syntax(ctx: &CheckContext, out: &mut Vec<Violation>) {
    traverse_for_errors(ctx.root, ctx.source, out);
}

fn traverse_for_errors(node: Node, source: &str, out: &mut Vec<Violation>) {
    if node.is_error() {
        report_error(node, source, out, "Syntax error detected");
    } else if node.is_missing() {
        report_error(node, source, out, format!("Missing expected node: {}", node.kind()));
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        traverse_for_errors(child, source, out);
    }
}

fn report_error(node: Node, _source: &str, out: &mut Vec<Violation>, msg: impl Into<String>) {
    let row = node.start_position().row + 1;
    out.push(Violation::simple(row, msg.into(), "LAW OF INTEGRITY"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::Lang;
    use crate::config::RuleConfig;
    use tree_sitter::Parser;

    #[test]
    fn test_rust_error() {
        let code = "fn main() { let x = ; }";
        let lang = Lang::Rust;
        let mut parser = Parser::new();
        parser.set_language(lang.grammar()).unwrap();
        let tree = parser.parse(code, None).unwrap();
        let config = RuleConfig::default();
        let ctx = CheckContext {
            root: tree.root_node(),
            source: code,
            filename: "test.rs",
            config: &config,
        };
        let mut violations = Vec::new();
        check_syntax(&ctx, &mut violations);
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_valid_rust() {
        let code = "fn main() { let x = 5; }";
        let lang = Lang::Rust;
        let mut parser = Parser::new();
        parser.set_language(lang.grammar()).unwrap();
        let tree = parser.parse(code, None).unwrap();
        let config = RuleConfig::default();
        let ctx = CheckContext {
            root: tree.root_node(),
            source: code,
            filename: "test.rs",
            config: &config,
        };
        let mut violations = Vec::new();
        check_syntax(&ctx, &mut violations);
        assert!(violations.is_empty());
    }
}
