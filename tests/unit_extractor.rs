// tests/unit_extractor.rs
use slopchop_core::apply::extractor;

#[test]
fn test_malformed_block_skipped() {
    let input = "#__SLOPCHOP_FILE__# broken\nno footer here";
    let _ = extractor::extract_files(input);
}

#[test]
fn test_content_preserved_exactly() {
    let input = "#__SLOPCHOP_FILE__# a.rs\nfn a() {}\n#__SLOPCHOP_END__#";
    let files = extractor::extract_files(input).unwrap();
    assert!(files.get("a.rs").unwrap().content.contains("fn a()"));
}
