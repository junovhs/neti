// src/analysis/v2/cognitive.rs
//! Cognitive Complexity metric implementation.
//! Based on the `SonarSource` specification.
//!
//! Key Rules:
//! 1. Increment for each break in linear flow (if, else, for, while, catch, etc.).
//! 2. Increment for nesting (each structure adds its nesting level to the score).
//! 3. Mitigations: `else if` does not increment nesting level.

use tree_sitter::Node;

pub struct CognitiveAnalyzer;

impl CognitiveAnalyzer {
    /// Calculates the Cognitive Complexity of a node.
    #[must_use]
    pub fn calculate(node: Node, source: &str) -> usize {
        let mut scorer = Scorer::new(source);
        scorer.visit(node, 0);
        scorer.score
    }
}

struct Scorer<'a> {
    source: &'a str,
    score: usize,
}

impl<'a> Scorer<'a> {
    fn new(source: &'a str) -> Self {
        Self { source, score: 0 }
    }

    fn visit(&mut self, node: Node, nesting: usize) {
        let (increment, next_nesting) = self.assess_node(node, nesting);
        self.score += increment;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit(child, next_nesting);
        }
    }

    fn assess_node(&self, node: Node, nesting: usize) -> (usize, usize) {
        let kind = node.kind();
        match kind {
            "if_expression" | "match_expression" | "for_expression" | "while_expression" | "loop_expression" => {
                Self::handle_control_flow(node, nesting)
            }
            "binary_expression" => (Self::handle_logic(node, self.source), nesting),
            "match_arm" => (1, nesting),
            "function_item" | "closure_expression" | "function_definition" | "method_definition" => (0, 0),
            _ => (0, nesting),
        }
    }

    fn handle_control_flow(node: Node, nesting: usize) -> (usize, usize) {
        if Self::is_else_if(node) {
            (1, nesting)
        } else {
            (1 + nesting, nesting + 1)
        }
    }

    fn handle_logic(node: Node, source: &str) -> usize {
        usize::from(Self::is_boolean_op(node, source))
    }

    fn is_else_if(node: Node) -> bool {
        let Some(parent) = node.parent() else {
            return false;
        };

        if parent.kind() == "if_expression" && Self::is_alternative_child(parent, node) {
            return true;
        }

        Self::has_else_sibling(node)
    }

    fn is_alternative_child(parent: Node, node: Node) -> bool {
        parent
            .child_by_field_name("alternative")
            .is_some_and(|alt| alt.id() == node.id())
    }

    fn has_else_sibling(node: Node) -> bool {
        let mut prev = node.prev_sibling();
        while let Some(s) = prev {
            if s.kind() == "else" {
                return true;
            }
            if s.is_named() {
                break;
            }
            prev = s.prev_sibling();
        }
        false
    }

    fn is_boolean_op(node: Node, source: &str) -> bool {
        let Some(op) = node.child_by_field_name("operator") else {
            return false;
        };

        op.utf8_text(source.as_bytes())
            .is_ok_and(|text| text == "&&" || text == "||")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse(code: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_rust::language())
            .expect("Failed to set language");
        parser.parse(code, None).expect("Failed to parse code")
    }

    #[test]
    fn test_linear_flow() {
        let code = "fn main() { let a = 1; let b = 2; }";
        let tree = parse(code);
        let score = CognitiveAnalyzer::calculate(tree.root_node(), code);
        assert_eq!(score, 0);
    }

    #[test]
    fn test_if_statement() {
        let code = "fn main() { if true { } }";
        let tree = parse(code);
        let score = CognitiveAnalyzer::calculate(tree.root_node(), code);
        assert_eq!(score, 1);
    }

    #[test]
    fn test_nested_if() {
        let code = "fn main() { if true { if true { } } }";
        let tree = parse(code);
        let score = CognitiveAnalyzer::calculate(tree.root_node(), code);
        assert_eq!(score, 3);
    }

    #[test]
    fn test_else_if_flattening() {
        let code = "fn main() { if true { } else if true { } }";
        let tree = parse(code);
        let score = CognitiveAnalyzer::calculate(tree.root_node(), code);
        assert_eq!(score, 2);
    }

    #[test]
    fn test_boolean_ops() {
        let code = "fn main() { if true && true { } }";
        let tree = parse(code);
        let score = CognitiveAnalyzer::calculate(tree.root_node(), code);
        assert_eq!(score, 2);
    }
}