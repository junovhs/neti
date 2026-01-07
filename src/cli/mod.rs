// src/cli/mod.rs
//! CLI command handlers.

pub mod args;
pub mod audit;
pub mod config_ui;
pub mod handlers;
pub mod locality;
pub mod mutate_handler;
pub mod stage_handlers;

pub use args::{Cli, Commands};
pub use handlers::{
    handle_apply, handle_check, handle_map, handle_pack, handle_scan, handle_signatures,
    handle_stage, PackArgs,
};
pub use mutate_handler::handle_mutate;

pub use audit::handle as handle_audit;
