// src/cli/mod.rs
//! CLI command handlers.

pub mod args;
pub mod audit;
pub mod handlers;
pub mod locality;

pub use args::{Cli, Commands};
pub use handlers::{
    handle_apply, handle_check, handle_map, handle_pack, handle_scan, handle_signatures, PackArgs,
};

pub use audit::handle as handle_audit;
