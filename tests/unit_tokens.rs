// tests/unit_tokens.rs
use warden_core::tokens::Tokenizer;

#[test]
fn test_tokenizer_available() {
    assert!(Tokenizer::is_available());
}

#[test]
fn test_count_basic() {
    let count = Tokenizer::count("Hello world");
    assert!((2..=5).contains(&count), "Expected 2-5 tokens, got {count}");
}

#[test]
fn test_exceeds_limit() {
    assert!(!Tokenizer::exceeds_limit("hi", 100));
    assert!(Tokenizer::exceeds_limit("hello world test", 1));
}

#[test]
fn test_fallback_returns_zero() {
    assert_eq!(Tokenizer::count(""), 0);
}
