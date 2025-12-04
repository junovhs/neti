// src/trace/mod.rs
//! The `slopchop trace` command - Smart context generation.

mod options;
mod output;
mod runner;

pub use options::TraceOptions;
pub use runner::{map, run, TraceResult};
