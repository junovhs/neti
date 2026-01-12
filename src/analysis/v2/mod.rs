// src/analysis/v2/mod.rs
pub mod cognitive;
pub mod engine;
pub mod inspector;
pub mod metrics;
pub mod patterns;
pub mod rust;
pub mod scope;
pub mod visitor;
pub(crate) mod worker;
pub(crate) mod aggregator;
pub(crate) mod deep;

pub use engine::ScanEngineV2;