use super::PatternTemplate;

/// Built-in patterns to detect.
///
/// These patterns identify ACTUAL consolidation opportunities - code that
/// could be meaningfully refactored to reduce duplication. They do NOT
/// flag fundamental Rust idioms like `format!` or `.collect()`.
///
/// Each pattern should have:
/// 1. A clear consolidation path (what to do about it)
/// 2. A minimum occurrence threshold that indicates real duplication
/// 3. Specificity - not just "you used the standard library"
pub const PATTERNS: &[PatternTemplate] = &[
    // ==========================================================
    // LOOP PATTERNS - Manual iteration that could be functional
    // ==========================================================
    PatternTemplate {
        name: "manual_vec_build",
        description: "Manual Vec building loop (consider .collect() or vec![])",
        rust_query: r#"
            (for_expression
                body: (block
                    (expression_statement
                        (call_expression
                            function: (field_expression
                                field: (field_identifier) @method
                                (#eq? @method "push"))))))
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    PatternTemplate {
        name: "manual_hashmap_build",
        description: "Manual HashMap building loop (consider .collect() into HashMap)",
        rust_query: r#"
            (for_expression
                body: (block
                    (expression_statement
                        (call_expression
                            function: (field_expression
                                field: (field_identifier) @method
                                (#eq? @method "insert"))))))
        "#,
        python_query: None,
        min_occurrences: 3,
    },
    // ==========================================================
    // NESTING PATTERNS - Deep nesting that could be flattened
    // ==========================================================
    PatternTemplate {
        name: "nested_if_let",
        description: "Nested if-let chains (consider let-else, ? operator, or early returns)",
        rust_query: r"
            (if_expression
                condition: (let_condition)
                consequence: (block
                    (expression_statement
                        (if_expression
                            condition: (let_condition)))))
        ",
        python_query: None,
        min_occurrences: 3,
    },
    PatternTemplate {
        name: "nested_match",
        description: "Nested match expressions (consider flattening with tuple matching)",
        rust_query: r"
            (match_expression
                body: (match_block
                    (match_arm
                        value: (match_expression))))
        ",
        python_query: None,
        min_occurrences: 3,
    },
    // ==========================================================
    // BOILERPLATE PATTERNS - Repeated ceremony
    // ==========================================================
    PatternTemplate {
        name: "impl_from_for_error",
        description: "Multiple From impls for error types (consider thiserror #[from])",
        rust_query: r#"
            (impl_item
                trait: (generic_type
                    type: (type_identifier) @trait
                    (#eq? @trait "From"))
                type: (type_identifier) @for_type
                body: (declaration_list
                    (function_item
                        name: (identifier) @fn_name
                        (#eq? @fn_name "from"))))
        "#,
        python_query: None,
        min_occurrences: 3,
    },
    PatternTemplate {
        name: "impl_display",
        description: "Multiple Display impls with similar structure (consider macro or derive)",
        rust_query: r#"
            (impl_item
                trait: (type_identifier) @trait
                (#eq? @trait "Display")
                body: (declaration_list
                    (function_item
                        name: (identifier) @fn
                        (#eq? @fn "fmt"))))
        "#,
        python_query: None,
        min_occurrences: 3,
    },
    // ==========================================================
    // FILE I/O PATTERNS - Common file operations
    // ==========================================================
    PatternTemplate {
        name: "repeated_file_read",
        description: "Repeated fs::read_to_string calls (consider a file reading helper)",
        rust_query: r#"
            (call_expression
                function: (scoped_identifier
                    path: (identifier) @mod
                    name: (identifier) @fn
                    (#eq? @mod "fs")
                    (#eq? @fn "read_to_string")))
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    PatternTemplate {
        name: "repeated_file_write",
        description: "Repeated fs::write calls (consider a file writing helper)",
        rust_query: r#"
            (call_expression
                function: (scoped_identifier
                    path: (identifier) @mod
                    name: (identifier) @fn
                    (#eq? @mod "fs")
                    (#eq? @fn "write")))
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    // ==========================================================
    // CONTROL FLOW - Verbose patterns
    // ==========================================================
    PatternTemplate {
        name: "long_if_else_chain",
        description: "Long if-else chain (consider match expression or lookup table)",
        rust_query: r"
            (if_expression
                alternative: (else_clause
                    (if_expression
                        alternative: (else_clause
                            (if_expression
                                alternative: (else_clause))))))
        ",
        python_query: None,
        min_occurrences: 2,
    },
    PatternTemplate {
        name: "repeated_early_return",
        description: "Repeated early return pattern (consider a validation helper)",
        rust_query: r#"
            (if_expression
                consequence: (block
                    (expression_statement
                        (return_expression
                            (call_expression
                                function: (identifier) @err
                                (#eq? @err "Err"))))))
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    // ==========================================================
    // STRING PATTERNS - Repeated string operations
    // ==========================================================
    PatternTemplate {
        name: "repeated_path_join",
        description: "Repeated path.join() chains (consider a path builder helper)",
        rust_query: r#"
            (call_expression
                function: (field_expression
                    value: (call_expression
                        function: (field_expression
                            field: (field_identifier) @inner
                            (#eq? @inner "join")))
                    field: (field_identifier) @outer
                    (#eq? @outer "join")))
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    // ==========================================================
    // TEST PATTERNS - Test boilerplate
    // ==========================================================
    PatternTemplate {
        name: "test_tempdir_setup",
        description: "Repeated tempdir test setup (consider a test fixture helper)",
        rust_query: r#"
            (let_declaration
                pattern: (identifier)
                value: (call_expression
                    function: (scoped_identifier
                        name: (identifier) @fn
                        (#match? @fn "^(tempdir|tempfile|TempDir)$"))))
        "#,
        python_query: None,
        min_occurrences: 5,
    },
];