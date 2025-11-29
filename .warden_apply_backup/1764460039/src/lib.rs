// src/lib.rs
pub mod analysis;
pub mod apply;
pub mod clipboard;
pub mod config;
pub mod constants;
pub mod discovery;
pub mod error;
pub mod pack;
pub mod project;
pub mod prompt;
pub mod reporting;
pub mod roadmap;
pub mod skeleton;
pub mod tokens;
pub mod tui;
pub mod types;

// Legacy/Test compatibility aliases
pub use analysis as rules;