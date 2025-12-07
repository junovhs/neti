// src/error.rs
//! Error handling - just use anyhow everywhere.
//!
//! This module exists for backward compatibility during migration.
//! New code should use `anyhow::Result` directly.

pub use anyhow::{anyhow, bail, Context, Error, Result};

// Legacy alias for gradual migration
pub type SlopChopError = anyhow::Error;
