//! Output formatting for consolidation audit reports.

pub mod ai;
pub mod json;
pub mod terminal;

pub use ai::format_ai_prompt;
pub use json::format_json;
pub use terminal::{format_opportunity_detail, format_terminal};
