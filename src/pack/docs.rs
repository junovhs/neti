// src/pack/docs.rs
//! Logic for extracting documentation from source code.

use crate::lang::Lang;
use anyhow::Result;
use std::fmt::Write;
use std::path::Path;

/// Extracts documentation comments from a file based on its language.
///
/// # Errors
/// Returns error if formatting fails.
pub fn extract_from_path(path: &Path, content: &str) -> Result<String> {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let lang = Lang::from_ext(ext);

    let mut docs = String::new();
    let lines: Vec<&str> = content.lines().collect();

    match lang {
         Some(Lang::Python) => extract_python_docs(lines, &mut docs)?,
         _ => extract_c_style_docs(lines, &mut docs)?,
    }
    
    Ok(docs)
}

fn extract_python_docs(lines: Vec<&str>, docs: &mut String) -> Result<()> {
    let mut inside_docstring = false;
    for line in lines {
        process_python_line(line, docs, &mut inside_docstring)?;
    }
    Ok(())
}

fn process_python_line(line: &str, docs: &mut String, inside: &mut bool) -> Result<()> {
    let trimmed = line.trim();
    if is_docstring_marker(trimmed) {
        if is_one_line_docstring(trimmed) {
            let text = &trimmed[3..trimmed.len()-3];
            if !text.is_empty() {
                writeln!(docs, "{text}")?;
            }
        } else {
            *inside = !*inside;
        }
        return Ok(());
    }
    
    if *inside {
        writeln!(docs, "{line}")?;
    }
    Ok(())
}

fn is_docstring_marker(s: &str) -> bool {
    s.starts_with("\"\"\"") || s.starts_with("'''")
}

fn is_one_line_docstring(s: &str) -> bool {
    s.len() > 3 && (s.ends_with("\"\"\"") || s.ends_with("'''"))
}

fn extract_c_style_docs(lines: Vec<&str>, docs: &mut String) -> Result<()> {
    for line in lines {
        process_c_style_line(line, docs)?;
    }
    Ok(())
}

fn process_c_style_line(line: &str, docs: &mut String) -> Result<()> {
    let trimmed = line.trim();
    if let Some(comment) = trimmed.strip_prefix("///") {
        let text = comment.strip_prefix(' ').unwrap_or(comment);
        writeln!(docs, "{text}")?;
        return Ok(());
    } 
    
    if let Some(comment) = trimmed.strip_prefix("/**") {
        let text = comment.trim_end_matches("*/").trim();
        if !text.is_empty() {
            writeln!(docs, "{text}")?;
        }
        return Ok(());
    }
    
    if trimmed.starts_with('*') && !trimmed.starts_with("*/") {
        let text = trimmed.trim_start_matches('*').trim();
        if !text.is_empty() {
            writeln!(docs, "{text}")?;
        }
    }
    Ok(())
}
