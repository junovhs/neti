// src/cli/mod.rs
//! CLI command handlers.

pub mod args;
pub mod audit;
pub mod config_ui;
pub mod dispatch;
pub mod handlers;
pub mod locality;
pub mod mutate_handler;

pub use args::Cli;
