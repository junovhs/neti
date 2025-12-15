// src/audit/patterns/registry_extra.rs
//! Additional pattern templates - loops, clones, closures, file I/O.

use super::PatternTemplate;

/// Extended patterns for loops, clones, closures, strings, and file I/O.
pub const EXTRA_PATTERNS: &[PatternTemplate] = &[
    // === LOOP PATTERNS ===
    PatternTemplate {
        name: "for_loop_push",
        description: "For loop with push (likely .map().collect() or .filter().collect())",
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
        name: "for_loop_insert",
        description: "For loop with insert (likely .collect::<HashMap>())",
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
    PatternTemplate {
        name: "for_loop_counter",
        description: "For loop with manual index (consider .enumerate())",
        rust_query: r"
            (for_expression
                body: (block
                    (expression_statement
                        (assignment_expression
                            left: (identifier)
                            right: (binary_expression)))))
        ",
        python_query: None,
        min_occurrences: 3,
    },
    // === TEST PATTERNS ===
    PatternTemplate {
        name: "test_assert_eq_pattern",
        description: "Repeated assert_eq! in tests (consider parameterized test helper)",
        rust_query: r#"
            (expression_statement
                (macro_invocation
                    macro: (identifier) @mac
                    (#eq? @mac "assert_eq")))
        "#,
        python_query: None,
        min_occurrences: 20,
    },
    PatternTemplate {
        name: "test_setup_let_binding",
        description: "Repeated let bindings at test start (consider test fixture)",
        rust_query: r#"
            (function_item
                (attribute_item
                    (attribute
                        (identifier) @attr
                        (#eq? @attr "test")))
                body: (block
                    (let_declaration)
                    (let_declaration)
                    (let_declaration)))
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    // === STRING PATTERNS ===
    PatternTemplate {
        name: "to_string_call",
        description: "Repeated .to_string() calls (consider Into<String> or Cow)",
        rust_query: r#"
            (call_expression
                function: (field_expression
                    field: (field_identifier) @method
                    (#eq? @method "to_string")))
        "#,
        python_query: None,
        min_occurrences: 15,
    },
    // === CLONE PATTERNS ===
    PatternTemplate {
        name: "clone_in_loop",
        description: "clone() inside loop body (consider borrowing or Rc/Arc)",
        rust_query: r#"
            (for_expression
                body: (block
                    (_
                        (call_expression
                            function: (field_expression
                                field: (field_identifier) @method
                                (#eq? @method "clone"))))))
        "#,
        python_query: None,
        min_occurrences: 3,
    },
    PatternTemplate {
        name: "clone_method_arg",
        description: "clone() as method argument (consider taking reference)",
        rust_query: r#"
            (arguments
                (call_expression
                    function: (field_expression
                        field: (field_identifier) @method
                        (#eq? @method "clone"))))
        "#,
        python_query: None,
        min_occurrences: 10,
    },
    // === CLOSURE PATTERNS ===
    PatternTemplate {
        name: "closure_map_ok",
        description: "Closure in map with Ok wrapping (consider map_ok or ok())",
        rust_query: r#"
            (call_expression
                function: (field_expression
                    field: (field_identifier) @method
                    (#eq? @method "map"))
                arguments: (arguments
                    (closure_expression
                        body: (call_expression
                            function: (identifier) @ok
                            (#eq? @ok "Ok")))))
        "#,
        python_query: None,
        min_occurrences: 3,
    },
    PatternTemplate {
        name: "closure_unwrap_or",
        description: "Repeated unwrap_or_else with closure (consider helper or Default)",
        rust_query: r#"
            (call_expression
                function: (field_expression
                    field: (field_identifier) @method
                    (#eq? @method "unwrap_or_else"))
                arguments: (arguments
                    (closure_expression)))
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    // === FILE I/O PATTERNS ===
    PatternTemplate {
        name: "fs_read_to_string",
        description: "Repeated fs::read_to_string (consider file helper with context)",
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
        name: "fs_write",
        description: "Repeated fs::write (consider file helper with context)",
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
    PatternTemplate {
        name: "path_join_chain",
        description: "Chained path.join().join() (consider path builder)",
        rust_query: r#"
            (call_expression
                function: (field_expression
                    value: (call_expression
                        function: (field_expression
                            field: (field_identifier) @j1
                            (#eq? @j1 "join")))
                    field: (field_identifier) @j2
                    (#eq? @j2 "join")))
        "#,
        python_query: None,
        min_occurrences: 5,
    },
];