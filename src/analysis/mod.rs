//! Core analysis logic (The "Rule Engine").

pub mod aggregator;
pub mod ast;
pub mod checks;
pub mod cognitive;
pub mod deep;
pub mod extract;
pub mod extract_impl; // New module
pub mod inspector;
pub mod metrics;
pub mod patterns;
pub mod safety;
pub mod scope;
pub mod structural;
pub mod visitor;
pub mod worker;

mod engine;

pub use aggregator::FileAnalysis;
pub use engine::Engine;
