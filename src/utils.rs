// src/utils.rs
use sha2::{Digest, Sha256};

/// Computes SHA256 hash of content with normalized line endings.
/// Always normalizes CRLF/CR to LF before hashing to ensure consistent
/// hashes across Windows/Unix platforms.
#[must_use]
pub fn compute_sha256(content: &str) -> String {
    let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    format!("{:x}", hasher.finalize())
}
