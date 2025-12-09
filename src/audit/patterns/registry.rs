use super::PatternTemplate;

/// Built-in patterns to detect.
pub const PATTERNS: &[PatternTemplate] = &[
    PatternTemplate {
        name: "process_spawn",
        description: "Process spawning with stdin pipe (spawn → pipe → wait)",
        rust_query: r#"
            (call_expression
                function: (scoped_identifier
                    path: (identifier) @_cmd (#eq? @_cmd "Command")
                    name: (identifier) @_new (#eq? @_new "new"))
            ) @spawn
        "#,
        python_query: None,
        min_occurrences: 3,
    },
    PatternTemplate {
        name: "option_map_chain",
        description: "Option chaining pattern (map/and_then/ok_or)",
        rust_query: r#"
            (call_expression
                function: (field_expression
                    field: (field_identifier) @method
                    (#match? @method "^(map|and_then|ok_or|unwrap_or)$"))
            ) @chain
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    PatternTemplate {
        name: "error_context",
        description: "Error context wrapping pattern (.context())",
        rust_query: r#"
            (call_expression
                function: (field_expression
                    field: (field_identifier) @method
                    (#eq? @method "context"))
            ) @context
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    PatternTemplate {
        name: "string_format",
        description: "String formatting with format!",
        rust_query: r#"
            (macro_invocation
                macro: (identifier) @name
                (#match? @name "^(format|println|eprintln|write|writeln)$")
            ) @format
        "#,
        python_query: None,
        min_occurrences: 10,
    },
    PatternTemplate {
        name: "impl_default",
        description: "Default trait implementation pattern",
        rust_query: r#"
            (impl_item
                trait: (type_identifier) @trait
                (#eq? @trait "Default")
            ) @impl
        "#,
        python_query: None,
        min_occurrences: 3,
    },
    PatternTemplate {
        name: "match_result",
        description: "Match on Result pattern",
        rust_query: r#"
            (match_expression
                value: (_)
                body: (match_block
                    (match_arm
                        pattern: (tuple_struct_pattern
                            type: (identifier) @variant
                            (#match? @variant "^(Ok|Err)$")))))
        "#,
        python_query: None,
        min_occurrences: 3,
    },
    PatternTemplate {
        name: "vec_collect",
        description: "Iterator collect into Vec pattern",
        rust_query: r#"
            (call_expression
                function: (field_expression
                    field: (field_identifier) @method
                    (#eq? @method "collect"))
            ) @collect
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    PatternTemplate {
        name: "closure_move",
        description: "Move closure pattern",
        rust_query: r#"
            (closure_expression
                "move"
            ) @closure
        "#,
        python_query: None,
        min_occurrences: 3,
    },
];
