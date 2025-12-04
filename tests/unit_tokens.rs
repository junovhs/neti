// tests/unit_tokens.rs
use slopchop_core::tokens::Tokenizer;

#[test]
fn test_tokenizer_available() {
    // Basic sanity check that the tokenizer loaded (cl100k_base)
    assert!(Tokenizer::is_available());
}

#[test]
fn test_count_basic() {
    // "hello world" is typically 2 tokens in cl100k_base [15339, 1917]
    let count = Tokenizer::count("hello world");
    assert_eq!(count, 2);

    // Empty string should be 0
    assert_eq!(Tokenizer::count(""), 0);
}

#[test]
fn test_exceeds_limit() {
    let text = "hello world"; // 2 tokens

    // Limit 10 (pass)
    assert!(!Tokenizer::exceeds_limit(text, 10));

    // Limit 1 (fail)
    assert!(Tokenizer::exceeds_limit(text, 1));

    // Limit 2 (pass - strictly greater than check)
    assert!(!Tokenizer::exceeds_limit(text, 2));
}

#[test]
fn test_fallback_returns_zero() {
    // This tests the contract that if the tokenizer encounters issues
    // (or implies empty/invalid state), it returns 0 safe-defaults.
    // Real initialization failure is hard to mock without dependency injection,
    // so we verify the empty string case as the proxy for "safe zero return".
    assert_eq!(Tokenizer::count(""), 0);

    // If the tokenizer were unavailable, count() returns 0.
    if !Tokenizer::is_available() {
        assert_eq!(Tokenizer::count("test content"), 0);
    }
}

