// tests/unit_tokens.rs
//! Unit tests for the Tokenizer module.
//!
//! VERIFICATION STRATEGY:
//! 1. Determinism: Verify exact token counts for known inputs using `cl100k_base`.
//! 2. Domain Relevance: Verify counting of code constructs (brackets, keywords).
//! 3. Edge Cases: Empty strings, Unicode, and exact limit boundaries.

use warden_core::tokens::Tokenizer;

#[test]
fn test_tokenizer_available() {
    // Critical system check: Tokenizer must load successfully.
    assert!(
        Tokenizer::is_available(),
        "Tokenizer failed to initialize (cl100k_base missing?)"
    );
}

#[test]
fn test_count_basic() {
    // "Hello" = 1, " world" = 1. Total = 2.
    // This asserts the tokenizer is actually cl100k_base and working deterministically.
    let count = Tokenizer::count("Hello world");
    assert_eq!(count, 2, "Expected 'Hello world' to be exactly 2 tokens");
}

#[test]
fn test_count_code_constructs() {
    // Verify handling of common syntax characters
    // "fn" " main" "()" " {" "}"
    let code = "fn main() {}";
    let count = Tokenizer::count(code);
    
    // cl100k_base typically encodes this as:
    // "fn" " main" "() {}" (or similar combination).
    // Actual test shows 4 tokens.
    assert_eq!(count, 4, "Unexpected token count for basic Rust fn");
}

#[test]
fn test_count_unicode() {
    // Emojis and multi-byte chars
    let text = "🚀"; 
    let count = Tokenizer::count(text);
    // 🚀 is usually 1 or 2 tokens depending on encoding
    assert!(count > 0, "Emoji should count as tokens");
    
    let mixed = "let x = '🦀';";
    assert!(Tokenizer::count(mixed) > 5);
}

#[test]
fn test_exceeds_limit_boundary() {
    // "a" is 1 token.
    let text = "a";
    let limit = 1;
    
    // Case 1: Count (1) == Limit (1) -> Should PASS (False)
    assert!(
        !Tokenizer::exceeds_limit(text, limit),
        "Count == Limit should NOT exceed"
    );

    // Case 2: Count (1) > Limit (0) -> Should FAIL (True)
    assert!(
        Tokenizer::exceeds_limit(text, 0),
        "Count > Limit SHOULD exceed"
    );
}

#[test]
fn test_empty_string_is_zero() {
    // An empty string must always be 0 tokens.
    assert_eq!(Tokenizer::count(""), 0);
}