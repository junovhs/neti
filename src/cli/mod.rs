// src/cli/mod.rs
//! CLI command handlers.

pub mod apply_handler;
pub mod args;
pub mod config_ui;
pub mod dispatch;
pub mod git_ops;
pub mod handlers;
pub mod locality;
pub mod mutate_handler;

pub use args::Cli;