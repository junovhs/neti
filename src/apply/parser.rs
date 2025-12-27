// src/apply/parser.rs
//! Strict parser for the `SlopChop` XSC7XSC protocol.
//!
//! Enforces block typing to prevent injection attacks where metadata headers
//! (like MANIFEST) could be misinterpreted as file paths.

use crate::apply::types::Block;
use anyhow::{anyhow, Result};
use regex::Regex;

const SIGIL: &str = "XSC7XSC";

/// Parses the full input string into a sequence of typed Blocks.
///
/// # Errors
/// Returns error if block structure is malformed or regex compilation fails.
pub fn parse(input: &str) -> Result<Vec<Block>> {
    let mut blocks = Vec::new();
    
    // Allow common markdown/AI prefixes: indentation, blockquotes (>), lists (-, *, 1., 1))
    // We capture this prefix so we can strip it from the body content.
    let prefix_pattern = r"(?P<prefix>[\t >\-\*\d\.\)]*)";
    
    let header_re = Regex::new(&format!(r"(?m)^{prefix_pattern}{SIGIL} (PLAN|MANIFEST|FILE|PATCH|META) {SIGIL}(?: (.+))?\s*$"))?;
    let footer_re = Regex::new(&format!(r"(?m)^{prefix_pattern}{SIGIL} END {SIGIL}\s*$"))?;

    let mut current_pos = 0;

    // Find the first header match
    while let Some(header_match) = header_re.find_at(input, current_pos) {
        let caps = header_re.captures(&input[header_match.start()..header_match.end()])
            .ok_or_else(|| anyhow!("Regex capture failed at {}", header_match.start()))?;

        // 1. Identify the Block Type and Argument
        let kind = caps.get(2).map_or("UNKNOWN", |m| m.as_str());
        let arg = caps.get(3).map(|m| m.as_str().trim().to_string());
        
        // 2. Identify the Transport Prefix (e.g. "> " or "  ")
        // We will use this to clean the body content.
        let prefix = caps.name("prefix").map_or("", |m| m.as_str());

        let content_start = header_match.end();
        
        // 3. Find corresponding footer
        let footer_match = footer_re.find_at(input, content_start)
            .ok_or_else(|| anyhow!("Unclosed block: {kind} at byte {content_start}"))?;

        // 4. Extract and Clean Content
        let raw_content = &input[content_start..footer_match.start()];
        let clean_content = clean_block_content(raw_content, prefix);

        let block = create_block(kind, arg, clean_content)?;
        blocks.push(block);

        current_pos = footer_match.end();
    }

    Ok(blocks)
}

/// Removes the transport prefix from every line of the content.
///
/// Handles robust stripping:
/// 1. Try exact prefix match (e.g., "> ").
/// 2. If that fails, try trimmed prefix (e.g., ">").
///    This handles cases where empty lines or code lines drop the trailing space.
fn clean_block_content(raw: &str, prefix: &str) -> String {
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
        stripped
    } else if !trimmed_prefix.is_empty() {
        line.strip_prefix(trimmed_prefix).unwrap_or(line)
    } else {
        line
    }
}

fn create_block(kind: &str, arg: Option<String>, content: String) -> Result<Block> {
    match kind {
        "PLAN" => Ok(Block::Plan(content)),
        "MANIFEST" => Ok(Block::Manifest(content)),
        "META" => Ok(Block::Meta(content)),
        "FILE" => {
            let path = arg.ok_or_else(|| anyhow!("FILE block missing path argument"))?;
            validate_path_keyword(&path)?;
            Ok(Block::File { path, content })
        },
        "PATCH" => {
            let path = arg.ok_or_else(|| anyhow!("PATCH block missing path argument"))?;
            validate_path_keyword(&path)?;
            Ok(Block::Patch { path, content })
        },
        _ => Err(anyhow!("Unknown block type: {kind}")),
    }
}

/// Ensures a file path is not a reserved keyword, preventing parser confusion.
fn validate_path_keyword(path: &str) -> Result<()> {
    let upper = path.to_uppercase();
    if matches!(upper.as_str(), "MANIFEST" | "PLAN" | "META" | "PATCH" | "FILE" | "END") {
        return Err(anyhow!("Invalid file path: '{path}' is a reserved keyword"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plan_and_manifest() -> Result<()> {
        let input = format!(
            "{SIGIL} PLAN {SIGIL}\nMy Plan\n{SIGIL} END {SIGIL}\n\
             {SIGIL} MANIFEST {SIGIL}\nfile.rs\n{SIGIL} END {SIGIL}"
        );
        let blocks = parse(&input)?;
        assert_eq!(blocks.len(), 2);
        match &blocks[0] {
            Block::Plan(c) => assert_eq!(c, "My Plan"),
            _ => panic!("Expected Plan"),
        }
        match &blocks[1] {
            Block::Manifest(c) => assert_eq!(c, "file.rs"),
            _ => panic!("Expected Manifest"),
        }
        Ok(())
    }

    #[test]
    fn test_parse_file_and_patch() -> Result<()> {
        let input = format!(
            "{SIGIL} FILE {SIGIL} src/main.rs\nfn main() {{}}\n{SIGIL} END {SIGIL}\n\
             {SIGIL} PATCH {SIGIL} lib.rs\nDIFF\n{SIGIL} END {SIGIL}"
        );
        let blocks = parse(&input)?;
        assert_eq!(blocks.len(), 2);
        match &blocks[0] {
            Block::File { path, content } => {
                assert_eq!(path, "src/main.rs");
                assert_eq!(content, "fn main() {}");
            }
            _ => panic!("Expected File"),
        }
        match &blocks[1] {
            Block::Patch { path, content } => {
                assert_eq!(path, "lib.rs");
                assert_eq!(content, "DIFF");
            }
            _ => panic!("Expected Patch"),
        }
        Ok(())
    }

    #[test]
    fn test_rejects_keyword_path() {
        let input = format!("{SIGIL} FILE {SIGIL} MANIFEST\ncontent\n{SIGIL} END {SIGIL}");
        let err = parse(&input).unwrap_err();
        assert!(err.to_string().contains("reserved keyword"));
    }

    #[test]
    fn test_tolerant_parsing() -> Result<()> {
        // Test with indentation and list markers
        let input = format!(
            "  {SIGIL} PLAN {SIGIL}\n  Plan\n  {SIGIL} END {SIGIL}\n\
             > {SIGIL} MANIFEST {SIGIL}\n> Man\n> {SIGIL} END {SIGIL}\n\
             - {SIGIL} FILE {SIGIL} f.rs\n- Code\n- {SIGIL} END {SIGIL}"
        );
        let blocks = parse(&input)?;
        assert_eq!(blocks.len(), 3);
        
        match &blocks[0] {
            Block::Plan(c) => assert_eq!(c, "Plan"),
            _ => panic!("Expected Plan"),
        }
        match &blocks[1] {
            Block::Manifest(c) => assert_eq!(c, "Man"),
            _ => panic!("Expected Manifest"),
        }
        match &blocks[2] {
            Block::File { path, content } => {
                assert_eq!(path, "f.rs");
                assert_eq!(content, "Code");
            }
            _ => panic!("Expected File"),
        }
        Ok(())
    }

    #[test]
    fn test_inconsistent_prefix_parsing() -> Result<()> {
        // Test where header has "> " but body has ">" (no space)
        let input = format!(
            "> {SIGIL} FILE {SIGIL} f.rs\n>code\n> {SIGIL} END {SIGIL}"
        );
        let blocks = parse(&input)?;
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::File { content, .. } => {
                assert_eq!(content, "code"); // Should strip ">" even if prefix captured "> "
            }
            _ => panic!("Expected File"),
        }
        Ok(())
    }
}