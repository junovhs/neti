//! Console output formatting for scan results.
//!
//! Violations are grouped and deduplicated by rule code. The first occurrence
//! of each rule shows the full educational block (analysis, why, fix, suppress).
//! Subsequent occurrences of the same rule show a compact one-liner with a
//! back-reference.

mod console;
mod guidance;
mod rich;
mod shared;

use anyhow::Result;

/// Prints a formatted scan report to stdout with confidence tiers and
/// deduplication.
///
/// # Errors
/// Returns error if formatting fails.
pub use console::print_report;

/// Builds a rich, multi-line report string without ANSI colors for file
/// logging. This matches the exact fidelity of the console output.
///
/// # Errors
/// Returns error if formatting fails.
pub use rich::build_rich_report;

/// Formats a report as a string (for embedding in context files).
///
/// # Errors
/// Returns error if formatting fails.
pub use rich::format_report_string;

/// Prints a serializable object as JSON to stdout.
///
/// # Errors
/// Returns error if serialization fails.
pub fn print_json<T: serde::Serialize>(data: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    println!("{json}");
    Ok(())
}
