use tree_sitter::{Query, QueryCursor};

/// usage of a function that needs to be updated
#[derive(Debug)]
pub struct CallSite {
    pub file_path: String,
    pub line: usize,
    pub original_text: String,
    // We might need byte offsets for precise rewriting
}

/// Finds all call sites of the given function name in the provided source code.
///
/// # Errors
/// Returns error if query compilation fails.
#[allow(clippy::needless_pass_by_value)] // query engines often need ownership or complex lifetimes
pub fn find_call_sites(
    source_code: &[u8],
    file_path: &str,
    function_name: &str,
    language: &str,
) -> Result<Vec<CallSite>, String> {
    // Basic query construction depending on language
    let query_str = match language {
        "rust" => format!("(call_expression function: (identifier) @name (#eq? @name \"{function_name}\"))"),
        "python" => format!("(call function: (identifier) @name (#eq? @name \"{function_name}\"))"),
        // Add more languages as needed
        _ => return Ok(Vec::new()),
    };

    let lang = match language {
         "rust" => tree_sitter_rust::language(),
         "python" => tree_sitter_python::language(),
         _ => return Ok(Vec::new()),
    };

    let query = Query::new(lang, &query_str).map_err(|e| e.to_string())?;
    let mut cursor = QueryCursor::new();
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(lang).map_err(|e| e.to_string())?;
    
    let tree = parser.parse(source_code, None).ok_or("Failed to parse source")?;
    let root = tree.root_node();

    let mut sites = Vec::new();
    
    for m in cursor.matches(&query, root, source_code) {
        process_match(&m, source_code, file_path, &mut sites);
    }

    Ok(sites)
}

fn process_match(
    m: &tree_sitter::QueryMatch<'_, '_>,
    source_code: &[u8],
    file_path: &str,
    sites: &mut Vec<CallSite>,
) {
    for capture in m.captures {
         // The capture is the function identifier. The parent is likely the call expression.
         if let Some(parent) = capture.node.parent() {
             let start_line = parent.start_position().row + 1;
             let text = parent.utf8_text(source_code).unwrap_or("").to_string();
             
             sites.push(CallSite {
                 file_path: file_path.to_string(),
                 line: start_line,
                 original_text: text,
             });
         }
    }
}
