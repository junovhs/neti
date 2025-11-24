use crate::checks::{self, CheckContext};
use crate::config::RuleConfig;
use crate::types::Violation; // Updated import
use tree_sitter::{Parser, Query};

pub struct Analyzer {
    // Rust
    rust_naming: Query,
    rust_safety: Query,
    rust_complexity: Query,
    rust_banned: Query,

    // JS/TS
    js_naming: Query,
    js_safety: Query,
    js_complexity: Query,

    // Python
    py_naming: Query,
    py_safety: Query,
    py_complexity: Query,
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer {
    /// Compiles Tree-sitter queries.
    ///
    /// # Panics
    ///
    /// Panics if the internal hardcoded queries are invalid.
    #[must_use]
    pub fn new() -> Self {
        Self {
            // --- RUST ---
            rust_naming: Query::new(
                tree_sitter_rust::language(),
                "(function_item name: (identifier) @name)",
            )
            .expect("Invalid Rust naming"),
            rust_safety: Query::new(
                tree_sitter_rust::language(),
                r#"
                (match_expression) @safe
                (if_expression condition: (let_condition)) @safe
                (while_expression condition: (let_condition)) @safe
                (try_expression) @safe
                (call_expression function: (field_expression field: (field_identifier) @m (#match? @m "^(expect|unwrap_or|unwrap_or_else|unwrap_or_default|ok|err|map_err|any|all|find|is_some|is_none|is_ok|is_err)$"))) @safe
                (function_item return_type: (_) @ret (#match? @ret "Result")) @safe
            "#,
            )
            .expect("Invalid Rust safety"),
            rust_complexity: Query::new(
                tree_sitter_rust::language(),
                r#"
                (if_expression) @branch
                (match_arm) @branch
                (while_expression) @branch
                (for_expression) @branch
                (binary_expression operator: ["&&" "||"]) @branch
            "#,
            )
            .expect("Invalid Rust complexity"),
            rust_banned: Query::new(
                tree_sitter_rust::language(),
                r#"
                (call_expression function: (field_expression field: (field_identifier) @m (#eq? @m "unwrap"))) @banned
            "#,
            )
            .expect("Invalid Rust banned"),

            // --- JS/TS ---
            js_naming: Query::new(
                tree_sitter_typescript::language_typescript(),
                r"
                (function_declaration name: (identifier) @name)
                (method_definition name: (property_identifier) @name)
                (variable_declarator name: (identifier) @name value: [(arrow_function) (function_expression)])
            ",
            )
            .expect("Invalid JS naming"),
            js_safety: Query::new(
                tree_sitter_typescript::language_typescript(),
                r#"
                (try_statement) @safe
                (call_expression function: (member_expression property: (property_identifier) @m (#eq? @m "catch"))) @safe
            "#,
            )
            .expect("Invalid JS safety"),
            js_complexity: Query::new(
                tree_sitter_typescript::language_typescript(),
                r#"
                (if_statement) @branch
                (for_statement) @branch
                (for_in_statement) @branch
                (while_statement) @branch
                (do_statement) @branch
                (switch_case) @branch
                (catch_clause) @branch
                (ternary_expression) @branch
                (binary_expression operator: ["&&" "||" "??"]) @branch
            "#,
            )
            .expect("Invalid JS complexity"),

            // --- PYTHON ---
            py_naming: Query::new(
                tree_sitter_python::language(),
                "(function_definition name: (identifier) @name)",
            )
            .expect("Invalid Py naming"),
            py_safety: Query::new(
                tree_sitter_python::language(),
                r#"
                (try_statement) @safe
                (if_statement condition: (unary_operator (_) @op (#eq? @op "not"))) @safe
                (if_statement condition: (comparison_operator (_) (none))) @safe
            "#,
            )
            .expect("Invalid Py safety"),
            py_complexity: Query::new(
                tree_sitter_python::language(),
                r"
                (if_statement) @branch
                (for_statement) @branch
                (while_statement) @branch
                (except_clause) @branch
                (boolean_operator) @branch
            ",
            )
            .expect("Invalid Py complexity"),
        }
    }

    /// Analyzes the content for violations.
    ///
    /// # Panics
    ///
    /// Panics if the Tree-sitter parser fails to initialize the language.
    #[must_use]
    pub fn analyze(
        &self,
        lang: &str,
        filename: &str,
        content: &str,
        config: &RuleConfig,
    ) -> Vec<Violation> {
        let (language, naming_q, safety_q, complexity_q, banned_q) = match lang {
            "rs" => (
                tree_sitter_rust::language(),
                &self.rust_naming,
                &self.rust_safety,
                &self.rust_complexity,
                Some(&self.rust_banned),
            ),
            "js" | "jsx" | "ts" | "tsx" => (
                tree_sitter_typescript::language_typescript(),
                &self.js_naming,
                &self.js_safety,
                &self.js_complexity,
                None,
            ),
            "py" => (
                tree_sitter_python::language(),
                &self.py_naming,
                &self.py_safety,
                &self.py_complexity,
                None,
            ),
            _ => return vec![],
        };

        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Failed to load language");
        let tree = parser.parse(content, None).expect("Failed to parse");
        let root = tree.root_node();

        // Create Context Object (Fixes Arity Violation)
        let ctx = CheckContext {
            root,
            source: content,
            filename,
            config,
        };

        let mut violations = Vec::new();

        let _ = checks::check_naming(&ctx, naming_q, &mut violations);
        let _ = checks::check_safety(&ctx, safety_q, &mut violations);
        let _ = checks::check_metrics(&ctx, complexity_q, &mut violations);

        if let Some(bq) = banned_q {
            let _ = checks::check_banned(&ctx, bq, &mut violations);
        }

        violations
    }
}
