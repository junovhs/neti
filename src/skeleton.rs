// src/skeleton.rs
use std::path::Path;

/// Reduces code to its structural skeleton (signatures only).
///
/// # Arguments
/// * `path` - The file path (used for language detection).
/// * `content` - The full source code.
///
/// # Returns
/// The skeletonized code, or the original content if language is unsupported.
#[must_use]
pub fn clean(path: &Path, content: &str) -> String {
    // Placeholder logic for v0.6.0 kickoff.
    // Real implementation will use Tree-sitter.
    // For now, if the flag is passed, we just return the content with a TODO marker
    // so we can verify the CLI wiring works.

    // Future: Match extension -> Select Query -> Execute
    let _ = path;
    format!("// SKELETONIZED (TODO: Implement Parser)\n{content}")
}
