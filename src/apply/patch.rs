// src/apply/patch.rs
//! Surgical patch application logic.
//!
//! Implements the "Search/Replace" block format.

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};

/// Applies a surgical patch to the original file content.
///
/// # Errors
/// Returns error if patch format is invalid, search block not found, or hash mismatch.
pub fn apply(original: &str, patch_content: &str) -> Result<String> {
    let (instructions, metadata) = parse_patch(patch_content)?;

    if let Some(expected_hash) = metadata.base_sha256 {
        verify_hash(original, &expected_hash)?;
    }

    let mut current_content = original.to_string();
    for instruction in instructions {
        current_content = apply_single_instruction(&current_content, instruction)?;
    }

    Ok(current_content)
}

fn verify_hash(content: &str, expected: &str) -> Result<()> {
    let actual = compute_sha256(content);
    if actual != expected {
        return Err(anyhow!(
            "Patch mismatch: Base SHA256 verification failed.\nExpected: {expected}\nActual:   {actual}"
        ));
    }
    Ok(())
}

struct PatchInstruction {
    search: String,
    replace: String,
}

struct PatchMetadata {
    base_sha256: Option<String>,
}

fn parse_patch(content: &str) -> Result<(Vec<PatchInstruction>, PatchMetadata)> {
    let mut instructions = Vec::new();
    let mut base_sha256 = None;

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        
        if let Some(stripped) = line.strip_prefix("BASE_SHA256:") {
            base_sha256 = Some(stripped.trim().to_string());
            i += 1;
            continue;
        }

        if line.trim() == "<<<< SEARCH" {
            let (instruction, next_i) = parse_block(&lines, i)?;
            instructions.push(instruction);
            i = next_i;
        } else {
            i += 1;
        }
    }

    if instructions.is_empty() {
        return Err(anyhow!("No valid <<<< SEARCH blocks found in patch."));
    }

    Ok((instructions, PatchMetadata { base_sha256 }))
}

fn parse_block(lines: &[&str], start_index: usize) -> Result<(PatchInstruction, usize)> {
    // Start after "<<<< SEARCH"
    let (search_text, i) = collect_section(
        lines,
        start_index + 1,
        "====",
        "Missing '====' separator after SEARCH block",
    )?;

    // Start after "===="
    let (replace_text, i) = collect_section(
        lines,
        i,
        ">>>>",
        "Missing '>>>>' terminator after REPLACE block",
    )?;

    Ok((
        PatchInstruction {
            search: search_text,
            replace: replace_text,
        },
        i,
    ))
}

fn collect_section(
    lines: &[&str],
    start_index: usize,
    terminator: &str,
    err_msg: &str,
) -> Result<(String, usize)> {
    let mut i = start_index;
    let mut collected = Vec::new();
    let mut found = false;

    while i < lines.len() {
        let line = lines[i];
        if line.trim() == terminator {
            found = true;
            i += 1; // Consume terminator
            break;
        }
        collected.push(line);
        i += 1;
    }

    if !found {
        return Err(anyhow!("{err_msg}"));
    }

    Ok((collected.join("\n"), i))
}

fn apply_single_instruction(content: &str, instr: PatchInstruction) -> Result<String> {
    let PatchInstruction { search, replace } = instr;
    let matches: Vec<_> = content.match_indices(&search).collect();

    match matches.len() {
        0 => Err(anyhow!("Patch failed: SEARCH block not found in file.\nHint: Check indentation and whitespace exactness.")),
        1 => Ok(content.replace(&search, &replace)),
        n => Err(anyhow!("Patch failed: Ambiguous SEARCH block. Found {n} matches.\nHint: Include more context in the SEARCH block to make it unique.")),
    }
}

fn compute_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_simple_replace() -> Result<()> {
        let original = "fn main() {\n    println!(\"Hello\");\n}";
        let patch = r#"
<<<< SEARCH
    println!("Hello");
====
    println!("World");
>>>>
"#;
        let result = apply(original, patch)?;
        assert_eq!(result, "fn main() {\n    println!(\"World\");\n}");
        Ok(())
    }

    #[test]
    fn test_apply_multiple_blocks() -> Result<()> {
        let original = "line1\nline2\nline3";
        let patch = r"
<<<< SEARCH
line1
====
LineOne
>>>>

<<<< SEARCH
line3
====
LineThree
>>>>
";
        let result = apply(original, patch)?;
        assert_eq!(result, "LineOne\nline2\nLineThree");
        Ok(())
    }

    #[test]
    fn test_fail_not_found() {
        let original = "content";
        let patch = r"
<<<< SEARCH
missing
====
new
>>>>
";
        if let Err(e) = apply(original, patch) {
            assert!(e.to_string().contains("not found"));
        } else {
            panic!("Should have failed with 'not found'");
        }
    }

    #[test]
    fn test_fail_ambiguous() {
        let original = "repeat\nrepeat";
        let patch = r"
<<<< SEARCH
repeat
====
fixed
>>>>
";
        if let Err(e) = apply(original, patch) {
            assert!(e.to_string().contains("Ambiguous"));
        } else {
            panic!("Should have failed with 'Ambiguous'");
        }
    }

    #[test]
    fn test_sha256_check() {
        let original = "secure data";
        let hash = compute_sha256(original);
        
        let valid_patch = format!("BASE_SHA256: {hash}\n<<<< SEARCH\ndata\n====\ninfo\n>>>>");
        assert!(apply(original, &valid_patch).is_ok());

        let invalid_patch = "BASE_SHA256: badhash\n<<<< SEARCH\ndata\n====\ninfo\n>>>>";
        if let Err(e) = apply(original, invalid_patch) {
            assert!(e.to_string().contains("Base SHA256 verification failed"));
        } else {
            panic!("Should have failed verification");
        }
    }
}