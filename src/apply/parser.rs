// src/apply/parser.rs
//! Strict parser for the `SlopChop` XSC7XSC protocol.
//!
//! Enforces block typing to prevent injection attacks where metadata headers
//! (like MANIFEST) could be misinterpreted as file paths.

use crate::apply::blocks::{clean_block_content, create_block};
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
    let (header_re, footer_re) = compile_patterns()?;
    let mut current_pos = 0;

    while let Some(header_match) = header_re.find_at(input, current_pos) {
        let (block, next_pos) = parse_single_block(input, &header_re, &footer_re, &header_match)?;
        blocks.push(block);
        current_pos = next_pos;
    }

    Ok(blocks)
}

fn compile_patterns() -> Result<(Regex, Regex)> {
    // Allow common markdown/AI prefixes: indentation, blockquotes, lists
    let prefix = r"(?P<prefix>[\t >\-\*\d\.\)\[\]]*)";

    let header = Regex::new(&format!(
        r"(?m)^{prefix}{SIGIL} (PLAN|MANIFEST|FILE|PATCH|META) {SIGIL}(?: (.+))?\s*$"
    ))?;
    let footer = Regex::new(&format!(r"(?m)^{prefix}{SIGIL} END {SIGIL}\s*$"))?;

    Ok((header, footer))
}

fn parse_single_block(
    input: &str,
    header_re: &Regex,
    _footer_re: &Regex,
    header_match: &regex::Match,
) -> Result<(Block, usize)> {
    let caps = header_re
        .captures(&input[header_match.start()..header_match.end()])
        .ok_or_else(|| anyhow!("Regex capture failed at {}", header_match.start()))?;

    let kind = caps.get(2).map_or("UNKNOWN", |m| m.as_str());
    let arg = caps.get(3).map(|m| m.as_str().trim().to_string());
    let prefix = caps.name("prefix").map_or("", |m| m.as_str());

    let content_start = header_match.end();
    
    // Fix I01: Bind footer to the specific prefix used by the header to prevent
    // content with different (or no) prefixes from terminating the block early.
    let escaped_prefix = regex::escape(prefix);
    let specific_footer_re = Regex::new(&format!(r"(?m)^{escaped_prefix}{SIGIL} END {SIGIL}\s*$"))?;

    let footer_match = specific_footer_re
        .find_at(input, content_start)
        .ok_or_else(|| anyhow!("Unclosed block: {kind} at byte {content_start}"))?;

    let raw_content = &input[content_start..footer_match.start()];
    let clean_content = clean_block_content(raw_content, prefix);

    let block = create_block(kind, arg, clean_content)?;
    Ok((block, footer_match.end()))
}


#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used)]
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
        assert!(matches!(&blocks[0], Block::Plan(c) if c == "My Plan"));
        assert!(matches!(&blocks[1], Block::Manifest(c) if c == "file.rs"));
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
        let input = format!(
            "  {SIGIL} PLAN {SIGIL}\n  Plan\n  {SIGIL} END {SIGIL}\n\
             > {SIGIL} MANIFEST {SIGIL}\n> Man\n> {SIGIL} END {SIGIL}\n\
             - {SIGIL} FILE {SIGIL} f.rs\n- Code\n- {SIGIL} END {SIGIL}"
        );
        let blocks = parse(&input)?;
        assert_eq!(blocks.len(), 3);
        assert!(matches!(&blocks[0], Block::Plan(c) if c == "Plan"));
        assert!(matches!(&blocks[1], Block::Manifest(c) if c == "Man"));
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
        let input = format!("> {SIGIL} FILE {SIGIL} f.rs\n>code\n> {SIGIL} END {SIGIL}");
        let blocks = parse(&input)?;
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::File { content, .. } => assert_eq!(content, "code"),
            _ => panic!("Expected File"),
        }
        Ok(())
    }
}
