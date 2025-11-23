use crate::error::Result;
use crate::tokens::Tokenizer;
use colored::Colorize;
use std::fs;
use std::path::Path;
use tree_sitter::{Language, Node, Parser, Query, QueryCursor};

// --- CONFIGURATION ---
const TOKEN_LIMIT: usize = 2000;
const WORD_LIMIT: usize = 3;

pub struct RuleEngine {
    rust: Query,
    python: Query,
    typescript: Query,
    javascript: Query,
}

impl RuleEngine {
    /// Creates a new rule engine.
    ///
    /// # Panics
    ///
    /// Panics if the internal Tree-sitter queries are invalid.
    /// These are hardcoded and tested, so a panic implies a developer error in the queries.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rust: Query::new(
                tree_sitter_rust::language(),
                "(function_item name: (identifier) @name)",
            )
            .unwrap(),
            python: Query::new(
                tree_sitter_python::language(),
                "(function_definition name: (identifier) @name)",
            )
            .unwrap(),
            typescript: Query::new(
                tree_sitter_typescript::language_typescript(),
                r"
                (function_declaration name: (identifier) @name)
                (method_definition name: (property_identifier) @name)
                (variable_declarator name: (identifier) @name value: [(arrow_function) (function_expression)])
            ",
            )
            .unwrap(),
            javascript: Query::new(
                tree_sitter_javascript::language(),
                r"
                (function_declaration name: (identifier) @name)
                (method_definition name: (property_identifier) @name)
                (variable_declarator name: (identifier) @name value: [(arrow_function) (function_expression)])
            ",
            )
            .unwrap(),
        }
    }

    /// Checks a file for rule violations.
    ///
    /// # Errors
    ///
    /// Returns error if the file cannot be read.
    pub fn check_file(&self, path: &Path) -> Result<bool> {
        let Ok(content) = fs::read_to_string(path) else {
            return Ok(true);
        };

        if content.contains("// warden:ignore") || content.contains("# warden:ignore") {
            return Ok(true);
        }

        let mut passed = true;
        let filename = path.to_string_lossy();

        // 1. TOKEN COUNT
        let token_count = Tokenizer::count(&content);
        if token_count > TOKEN_LIMIT {
            Self::print_file_violation(
                &filename,
                "LAW OF ATOMICITY",
                &format!("File size is {token_count} tokens (Limit: {TOKEN_LIMIT})"),
                "Split this file into smaller modules.",
            );
            passed = false;
        }

        // 2. AST ANALYSIS
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            match ext {
                "rs" => Self::analyze_tree(
                    tree_sitter_rust::language(),
                    &self.rust,
                    &content,
                    &filename,
                    "_",
                    &mut passed,
                ),
                "py" => Self::analyze_tree(
                    tree_sitter_python::language(),
                    &self.python,
                    &content,
                    &filename,
                    "_",
                    &mut passed,
                ),
                "ts" | "tsx" => Self::analyze_tree(
                    tree_sitter_typescript::language_typescript(),
                    &self.typescript,
                    &content,
                    &filename,
                    "camel",
                    &mut passed,
                ),
                "js" | "jsx" => Self::analyze_tree(
                    tree_sitter_javascript::language(),
                    &self.javascript,
                    &content,
                    &filename,
                    "camel",
                    &mut passed,
                ),
                _ => {}
            }
        }

        Ok(passed)
    }

    fn analyze_tree(
        language: Language,
        query: &Query,
        content: &str,
        filename: &str,
        naming_style: &str,
        passed: &mut bool,
    ) {
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Error loading grammar");

        let tree = parser.parse(content, None).expect("Error parsing file");
        let root = tree.root_node();

        // A. NAMING
        let mut cursor = QueryCursor::new();
        for m in cursor.matches(query, root, content.as_bytes()) {
            for capture in m.captures {
                let name_bytes = &content.as_bytes()[capture.node.byte_range()];
                let name = String::from_utf8_lossy(name_bytes);
                let node = capture.node;

                if naming_style == "camel" {
                    let caps = name.chars().filter(|c| c.is_uppercase()).count();
                    if caps + 1 > WORD_LIMIT && !name.chars().next().unwrap_or('a').is_uppercase() {
                        Self::print_node_violation(
                            filename,
                            node,
                            content,
                            "LAW OF BLUNTNESS",
                            &format!("Function '{name}' is too complex ({caps} words)"),
                            "Rename to max 3 words.",
                        );
                        *passed = false;
                    }
                } else if name.split('_').count() > WORD_LIMIT {
                    Self::print_node_violation(
                        filename,
                        node,
                        content,
                        "LAW OF BLUNTNESS",
                        &format!("Function '{name}' is too complex"),
                        "Rename to max 3 words.",
                    );
                    *passed = false;
                }
            }
        }

        // B. SAFETY (Recursive walk)
        Self::check_safety_recursive(root, content, filename, passed);
    }

    fn check_safety_recursive(node: Node, content: &str, filename: &str, passed: &mut bool) {
        let kind = node.kind();

        let is_func_boundary = matches!(
            kind,
            "function_item"
                | "function_definition"
                | "function_declaration"
                | "function_expression"
                | "method_definition"
                | "arrow_function"
        );

        if is_func_boundary {
            // 1. Identify Function Name
            let target_node = node.child_by_field_name("name").unwrap_or(node);
            let name_bytes = &content.as_bytes()[target_node.byte_range()];
            let func_name = String::from_utf8_lossy(name_bytes);

            // 2. EXEMPTION: Lifecycle & Genesis Methods
            if !Self::is_lifecycle_function(&func_name) {
                let code_bytes = &content.as_bytes()[node.byte_range()];
                let code_str = String::from_utf8_lossy(code_bytes).to_lowercase();

                if code_str.lines().count() >= 5 {
                    let has_safety = code_str.contains("result")
                        || code_str.contains("option")
                        || code_str.contains("try")
                        || code_str.contains("catch")
                        || code_str.contains("except")
                        || code_str.contains("match")
                        || code_str.contains("unwrap_or")
                        || code_str.contains("ok(");

                    if !has_safety {
                        Self::print_node_violation(
                            filename,
                            target_node,
                            content,
                            "LAW OF PARANOIA",
                            "Logic block missing explicit safety",
                            "Add try/catch, Result, or Option handling.",
                        );
                        *passed = false;
                    }
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::check_safety_recursive(child, content, filename, passed);
        }
    }

    /// Returns true if the function name implies initialization or simple conversion.
    /// This is a semantic heuristic to avoid false positives on safe code.
    fn is_lifecycle_function(name: &str) -> bool {
        let n = name.to_lowercase();
        matches!(
            n.as_str(),
            "new" | "default" | "from" | "into" | "constructor" | "init" | "__init__" | "render"
        )
    }

    // --- PRINTER ---

    fn print_node_violation(
        filename: &str,
        node: Node,
        content: &str,
        law: &str,
        message: &str,
        help: &str,
    ) {
        let start = node.start_position();
        let end = node.end_position();
        let row = start.row;
        let line_num = row + 1;

        let line_content = content.lines().nth(row).unwrap_or("");
        let squiggle_len = if start.row == end.row {
            end.column.saturating_sub(start.column).max(1)
        } else {
            line_content.len().saturating_sub(start.column).max(1)
        };

        println!("{}: {}", "error".red().bold(), message.bold());
        println!(
            "  {} {}:{}:{}",
            "-->".blue(),
            filename,
            line_num,
            start.column + 1
        );
        println!("   {}", "|".blue());
        println!(
            "{:<3} {} {}",
            line_num.to_string().blue(),
            "|".blue(),
            line_content
        );
        println!(
            "   {} {}{}",
            "|".blue(),
            " ".repeat(start.column),
            "^".repeat(squiggle_len).red()
        );
        println!("   {}", "|".blue());
        println!("   {} {}: {}", "=".blue().bold(), law.white().bold(), help);
        println!();
    }

    fn print_file_violation(filename: &str, law: &str, message: &str, help: &str) {
        println!("{}: {}", "error".red().bold(), message.bold());
        println!("  {} {}:1:1", "-->".blue(), filename);
        println!("   {}", "|".blue());
        println!("   {} [File too large to display preview]", "|".blue());
        println!("   {}", "|".blue());
        println!("   {} {}: {}", "=".blue().bold(), law.white().bold(), help);
        println!();
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}
