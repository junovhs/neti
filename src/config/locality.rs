// src/config/locality.rs
//! Configuration for the Law of Locality enforcement.

use serde::{Deserialize, Serialize};


use crate::graph::locality::{ClassifierConfig, ValidatorConfig};

/// Locality rules configuration from slopchop.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LocalityConfig {
    /// Maximum D for non-Hub dependencies. Default: 4
    pub max_distance: usize,
    /// Threshold for L1 cache (always pass). Default: 2
    pub l1_threshold: usize,
    /// Minimum K (skew) to qualify as Hub. Default: 1.0
    pub hub_threshold: f64,
    /// Minimum Cₐ for Hub status. Default: 5
    pub min_hub_afferent: usize,
    /// Cₐ AND Cₑ above this = God Module. Default: 20
    pub god_module_threshold: usize,
    /// Cₐ + Cₑ below this = Deadwood. Default: 2
    pub deadwood_threshold: usize,
    /// Enforcement mode: "error", "warn", or "off"
    pub mode: String,
    /// Glob patterns to exempt from checks
    #[serde(default)]
    pub exempt_patterns: Vec<String>,
}

impl Default for LocalityConfig {
    fn default() -> Self {
        Self {
            max_distance: 4,
            l1_threshold: 2,
            hub_threshold: 1.0,
            min_hub_afferent: 5,
            god_module_threshold: 20,
            deadwood_threshold: 2,
            mode: "warn".to_string(),
            exempt_patterns: Vec::new(),
        }
    }
}

impl LocalityConfig {
    /// Converts to the validator's config format.
    #[must_use]
    pub fn to_validator_config(&self) -> ValidatorConfig {
        ValidatorConfig {
            max_distance: self.max_distance,
            l1_threshold: self.l1_threshold,
            classifier: ClassifierConfig {
                hub_threshold: self.hub_threshold,
                min_hub_afferent: self.min_hub_afferent,
                god_module_threshold: self.god_module_threshold,
                deadwood_threshold: self.deadwood_threshold,
                volatile_leaf_efferent: 5,
            },
            exempt_patterns: self.exempt_patterns.clone(),
        }
    }

    /// Returns true if enforcement is enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.mode != "off"
    }

    /// Returns true if violations should block.
    #[must_use]
    pub fn is_error_mode(&self) -> bool {
        self.mode == "error"
    }
}