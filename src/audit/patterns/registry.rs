// src/audit/patterns/registry.rs
//! Built-in pattern templates for consolidation detection.

use super::PatternTemplate;

pub use super::registry_extra::EXTRA_PATTERNS;

/// Core patterns - struct, error, and match patterns.
pub const PATTERNS: &[PatternTemplate] = &[
    // === STRUCT PATTERNS ===
    PatternTemplate {
        name: "struct_literal_many_fields",
        description: "Struct literal with 5+ fields (consider Default + struct update syntax)",
        rust_query: r"
            (struct_expression
                body: (field_initializer_list
                    (field_initializer)
                    (field_initializer)
                    (field_initializer)
                    (field_initializer)
                    (field_initializer)))
        ",
        python_query: None,
        min_occurrences: 5,
    },
    PatternTemplate {
        name: "option_field_none_init",
        description: "Struct with multiple None field inits (implement Default)",
        rust_query: r#"
            (field_initializer
                value: (identifier) @val
                (#eq? @val "None"))
        "#,
        python_query: None,
        min_occurrences: 10,
    },
    // === ERROR HANDLING ===
    PatternTemplate {
        name: "anyhow_bail_format",
        description: "Repeated bail!/anyhow! with format strings (consider error enum)",
        rust_query: r#"
            (macro_invocation
                macro: (identifier) @mac
                (#match? @mac "^(bail|anyhow)$"))
        "#,
        python_query: None,
        min_occurrences: 8,
    },
    PatternTemplate {
        name: "err_return_format",
        description: "Repeated Err(format!(...)) returns (consider typed errors)",
        rust_query: r#"
            (return_expression
                (call_expression
                    function: (identifier) @err
                    (#eq? @err "Err")
                    arguments: (arguments
                        (macro_invocation
                            macro: (identifier) @fmt
                            (#eq? @fmt "format")))))
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    // === MATCH PATTERNS ===
    PatternTemplate {
        name: "match_with_many_arms",
        description: "Match with 6+ arms (consider lookup table or trait dispatch)",
        rust_query: r"
            (match_expression
                body: (match_block
                    (match_arm) (match_arm) (match_arm)
                    (match_arm) (match_arm) (match_arm)))
        ",
        python_query: None,
        min_occurrences: 3,
    },
    PatternTemplate {
        name: "match_arm_same_body",
        description: "Match arms with trivial bodies (consider combining with |)",
        rust_query: r"
            (match_arm
                pattern: (or_pattern))
        ",
        python_query: None,
        min_occurrences: 5,
    },
    PatternTemplate {
        name: "if_let_some_pattern",
        description: "Repeated if-let Some(x) pattern (consider ? or map)",
        rust_query: r#"
            (if_expression
                condition: (let_condition
                    pattern: (tuple_struct_pattern
                        type: (identifier) @typ
                        (#eq? @typ "Some"))))
        "#,
        python_query: None,
        min_occurrences: 8,
    },
    // === IMPL PATTERNS ===
    PatternTemplate {
        name: "impl_from_manual",
        description: "Manual From impl (consider derive_more or thiserror #[from])",
        rust_query: r#"
            (impl_item
                trait: (generic_type
                    type: (type_identifier) @trait
                    (#eq? @trait "From"))
                body: (declaration_list
                    (function_item
                        name: (identifier) @fn
                        (#eq? @fn "from"))))
        "#,
        python_query: None,
        min_occurrences: 3,
    },
    PatternTemplate {
        name: "impl_display_write",
        description: "Display impl with write!/writeln! (consider derive_more Display)",
        rust_query: r#"
            (impl_item
                trait: (type_identifier) @trait
                (#eq? @trait "Display")
                body: (declaration_list
                    (function_item
                        body: (block
                            (expression_statement
                                (macro_invocation
                                    macro: (identifier) @mac
                                    (#match? @mac "^write")))))))
        "#,
        python_query: None,
        min_occurrences: 3,
    },
    // === NESTING PATTERNS ===
    PatternTemplate {
        name: "nested_if_else",
        description: "Deeply nested if-else (consider early returns or match)",
        rust_query: r"
            (if_expression
                alternative: (else_clause
                    (if_expression
                        alternative: (else_clause
                            (if_expression)))))
        ",
        python_query: None,
        min_occurrences: 2,
    },
    PatternTemplate {
        name: "triple_nested_block",
        description: "Triple-nested blocks (refactor to extract functions)",
        rust_query: r"
            (block
                (expression_statement
                    (_
                        (block
                            (expression_statement
                                (_
                                    (block)))))))
        ",
        python_query: None,
        min_occurrences: 2,
    },
];