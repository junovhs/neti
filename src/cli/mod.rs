// src/cli/mod.rs
//! CLI command handlers.

pub mod handlers;

pub use handlers::{
    handle_apply, handle_check, handle_dashboard, handle_fix, handle_map,
    handle_pack, handle_prompt, handle_trace, PackArgs,
};