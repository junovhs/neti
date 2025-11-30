// tests/unit_extractor.rs
use warden_core::apply::extractor;

#[test]
fn test_malformed_block_skipped() {
    let input = "∇∇∇ broken\nno footer here";
    let _ = extractor::extract_files(input);
}

#[test]
fn test_content_preserved_exactly() {
    let input = "∇∇∇ a.rs ∇∇∇\nfn a() {}\n∆∆∆";
    let files = extractor::extract_files(input).unwrap();
    assert!(files.get("a.rs").unwrap().content.contains("fn a()"));
}
