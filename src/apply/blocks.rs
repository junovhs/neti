// src/apply/blocks.rs
//! Block creation and content sanitization for the XSC7XSC protocol.

use crate::apply::types::Block;
use anyhow::{anyhow, Result};

/// Creates a typed Block from parsed components.
///
/// # Errors
/// Returns error if block type is unknown or required arguments are missing.
pub fn create_block(kind: &str, arg: Option<String>, content: String) -> Result<Block> {
    match kind {
        "PLAN" => Ok(Block::Plan(content)),
        "MANIFEST" => Ok(Block::Manifest(content)),
        "META" => Ok(Block::Meta(content)),
        "FILE" => create_file_block(arg, content),
        "PATCH" => create_patch_block(arg, content),
        _ => Err(anyhow!("Unknown block type: {kind}")),
    }
}

fn create_file_block(arg: Option<String>, content: String) -> Result<Block> {
    let arg_str = arg.ok_or_else(|| anyhow!("FILE block missing path argument"))?;
    let path = extract_path(&arg_str);
    validate_path_keyword(&path)?;
    Ok(Block::File { path, content })
}

fn create_patch_block(arg: Option<String>, content: String) -> Result<Block> {
    let path = arg.ok_or_else(|| anyhow!("PATCH block missing path argument"))?;
    validate_path_keyword(&path)?;
    Ok(Block::Patch { path, content })
}

/// Extracts the file path from an argument string, ignoring tags like SHA256 or [SKELETON].
fn extract_path(arg: &str) -> String {
    arg.split_whitespace().next().unwrap_or(arg).to_string()
}

/// Ensures a file path is not a reserved keyword.
///
/// # Errors
/// Returns error if the path matches a protocol keyword.
pub fn validate_path_keyword(path: &str) -> Result<()> {
    let upper = path.to_uppercase();
    let reserved = ["MANIFEST", "PLAN", "META", "PATCH", "FILE", "END"];
    if reserved.contains(&upper.as_str()) {
        return Err(anyhow!("Invalid file path: '{path}' is a reserved keyword"));
    }
    Ok(())
}

/// Removes the transport prefix from every line of block content.
///
/// Handles robust stripping for markdown/AI prefixes (>, -, *, etc.):
/// 1. Try exact prefix match (e.g., "> ").
/// 2. If that fails, try trimmed prefix (e.g., ">").
#[must_use]
pub fn clean_block_content(raw: &str, prefix: &str) -> String {
    if prefix.is_empty() {
        return raw.trim_matches('\n').to_string();
    }

    let trimmed_prefix = prefix.trim_end();
    let lines = raw.strip_prefix('\n').unwrap_or(raw).lines();

    lines
        .map(|line| clean_line(line, prefix, trimmed_prefix))
        .collect::<Vec<_>>()
        .join("\n")
}

fn clean_line<'a>(line: &'a str, prefix: &str, trimmed_prefix: &str) -> &'a str {
    if let Some(stripped) = line.strip_prefix(prefix) {
        return stripped;
    }
    if !trimmed_prefix.is_empty() {
        return line.strip_prefix(trimmed_prefix).unwrap_or(line);
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_block_plan() {
        let block = create_block("PLAN", None, "My plan".into()).unwrap();
        assert!(matches!(block, Block::Plan(c) if c == "My plan"));
    }

    #[test]
    fn test_create_block_file() {
        let block = create_block("FILE", Some("src/main.rs".into()), "code".into()).unwrap();
        match block {
            Block::File { path, content } => {
                assert_eq!(path, "src/main.rs");
                assert_eq!(content, "code");
            }
            _ => panic!("Expected File block"),
        }
    }

    #[test]
    fn test_validate_rejects_keywords() {
        assert!(validate_path_keyword("MANIFEST").is_err());
        assert!(validate_path_keyword("plan").is_err());
        assert!(validate_path_keyword("src/main.rs").is_ok());
    }

    #[test]
    fn test_clean_block_content_no_prefix() {
        let result = clean_block_content("\ncode\nmore\n", "");
        assert_eq!(result, "code\nmore");
    }

    #[test]
    fn test_clean_block_content_with_prefix() {
        let result = clean_block_content("\n> line1\n> line2", "> ");
        assert_eq!(result, "line1\nline2");
    }

    #[test]
    fn test_clean_line_trimmed_fallback() {
        // When prefix is "> " but line has just ">"
        let result = clean_line(">code", "> ", ">");
        assert_eq!(result, "code");
    }
}
