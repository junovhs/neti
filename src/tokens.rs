// src/tokens.rs
use std::sync::LazyLock;
use tiktoken_rs::CoreBPE;

/// The tokenizer encoding (`cl100k_base`, used by GPT-4/3.5-turbo).
/// Initialization is deferred until first use. If the encoding fails to load
/// (which should never happen with a valid tiktoken-rs installation),
/// token counting will return 0 and log an error.
static BPE: LazyLock<Option<CoreBPE>> = LazyLock::new(|| {
    tiktoken_rs::cl100k_base()
        .map_err(|e| eprintln!("Failed to load cl100k_base tokenizer: {e}"))
        .ok()
});

pub struct Tokenizer;

impl Tokenizer {
    /// Counts the number of tokens in the given text.
    /// Returns 0 if the tokenizer failed to initialize.
    #[must_use]
    pub fn count(text: &str) -> usize {
        BPE.as_ref()
            .map_or(0, |bpe| bpe.encode_ordinary(text).len())
    }

    /// Returns true if the text exceeds the token limit.
    #[must_use]
    pub fn exceeds_limit(text: &str, limit: usize) -> bool {
        Self::count(text) > limit
    }

    /// Returns true if the tokenizer is available.
    #[must_use]
    pub fn is_available() -> bool {
        BPE.is_some()
    }

    /// TEST FUNCTION TWO
    #[must_use]
    pub fn test_two() -> bool {
        false
    }
}