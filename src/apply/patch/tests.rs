// src/apply/patch/tests.rs
#![allow(clippy::indexing_slicing, clippy::unwrap_used)]

use super::*;

#[test]
fn test_v1_parse() -> Result<()> {
    let patch = "LEFT_CTX:\nfn foo() {\nOLD:\n    print(1);\nRIGHT_CTX:\n}\nNEW:\n    print(2);";
    let (instrs, meta) = parse_patch(patch)?;
    assert_eq!(instrs.len(), 1);
    assert!(matches!(meta.format, PatchFormat::V1));
    
    let i = &instrs[0];
    // collect_until_keyword adds newline
    assert!(i.search.contains("fn foo() {"));
    assert!(i.search.contains("print(1);"));
    assert!(i.replace.contains("print(2);"));
    Ok(())
}

#[test]
fn test_v1_apply() -> Result<()> {
    let original = "fn foo() {\n    print(1);\n}\n";
    let hash = compute_sha256(original);
    
    // V1 requires BASE_SHA256
    let patch = format!("BASE_SHA256: {hash}\nLEFT_CTX:\nfn foo() {{\nOLD:\n    print(1);\nRIGHT_CTX:\n}}\nNEW:\n    print(2);");
    
    let res = apply(original, &patch)?;
    assert_eq!(res, "fn foo() {\n    print(2);\n}\n");
    Ok(())
}

#[test]
fn test_diagnostic_ambiguous() {
    let original = "repeat\nrepeat\nrepeat";
    let patch = "<<<< SEARCH\nrepeat\n====\nfixed\n>>>>";
    let err = apply(original, patch).unwrap_err();
    assert!(err.to_string().contains("Found 3 occurrences"));
    assert!(err.to_string().contains("Line 1"));
}

#[test]
fn test_diagnostic_zero_match_probe() {
    // Original has long enough context to trigger probe logic
    let original = "fn main() { // start context is sufficiently long to be unique and detectable\n    let x = 100000000000;\n} // end context must also be long enough to be found";
    
    // Patch expects x = 5, but we ensure context is >20 chars so probe finds it
    let left = "fn main() { // start context is sufficiently long to be unique and detectable\n    let x = ";
    let right = ";\n} // end context must also be long enough to be found";
    
    // We artificially construct a patch that has enough chars to trigger probe
    // The search string must be > 40 chars
    let search = format!("{left}5555555555555555{right}"); 
    let replace = format!("{left}2{right}");
    
    // Manual instruction construction to test diagnostic logic directly
    let instr = PatchInstruction {
        search: search.clone(),
        replace,
        context_left: Some(left.to_string()),
    };
    
    let diag = diagnose_zero_matches(original, &search, &instr);
    
    // Verify we found the candidate region with the actual value (10000...)
    assert!(diag.contains("Did you mean this region?"));
    assert!(diag.contains("100000000000"));
}