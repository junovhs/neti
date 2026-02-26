// src/graph/locality/classifier.rs
//! Node classification based on coupling metrics.

use super::types::{Coupling, NodeIdentity};

/// Configuration thresholds for node classification.
#[derive(Debug, Clone)]
pub struct ClassifierConfig {
    pub hub_threshold: f64,
    pub min_hub_afferent: usize,
    pub god_module_threshold: usize,
    pub deadwood_threshold: usize,
    pub volatile_leaf_efferent: usize,
}

impl Default for ClassifierConfig {
    fn default() -> Self {
        Self {
            hub_threshold: 1.0,
            min_hub_afferent: 5,
            god_module_threshold: 20,
            deadwood_threshold: 2,
            volatile_leaf_efferent: 5,
        }
    }
}

/// Classifies a file based on its coupling metrics.
#[must_use]
pub fn classify(coupling: &Coupling, config: &ClassifierConfig) -> NodeIdentity {
    if is_god_module(coupling, config) {
        return NodeIdentity::GodModule;
    }
    if is_deadwood(coupling, config) {
        return NodeIdentity::IsolatedDeadwood;
    }
    if is_stable_hub(coupling, config) {
        return NodeIdentity::StableHub;
    }
    if is_volatile_leaf(coupling, config) {
        return NodeIdentity::VolatileLeaf;
    }
    NodeIdentity::Standard
}

fn is_god_module(coupling: &Coupling, config: &ClassifierConfig) -> bool {
    coupling.afferent() > config.god_module_threshold
        && coupling.efferent() > config.god_module_threshold
}

fn is_deadwood(coupling: &Coupling, config: &ClassifierConfig) -> bool {
    coupling.total() < config.deadwood_threshold
}

fn is_stable_hub(coupling: &Coupling, _config: &ClassifierConfig) -> bool {
    // Locality v2: Auto-detect hubs based on metrics.
    // High fan-in (>= 3) is the primary signal for a shared dependency.
    // We rely on is_god_module() (checked earlier) to filter out true monoliths.
    coupling.afferent() >= 3
}

fn is_volatile_leaf(coupling: &Coupling, config: &ClassifierConfig) -> bool {
    coupling.skew() < 0.0 && coupling.efferent() >= config.volatile_leaf_efferent
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stable_hub() {
        let config = ClassifierConfig::default();
        let coupling = Coupling::new(50, 2);
        assert_eq!(classify(&coupling, &config), NodeIdentity::StableHub);
    }

    #[test]
    fn test_volatile_leaf() {
        let config = ClassifierConfig::default();
        let coupling = Coupling::new(1, 12);
        assert_eq!(classify(&coupling, &config), NodeIdentity::VolatileLeaf);
    }

    #[test]
    fn test_god_module() {
        let config = ClassifierConfig::default();
        let coupling = Coupling::new(25, 25);
        assert_eq!(classify(&coupling, &config), NodeIdentity::GodModule);
    }

    #[test]
    fn test_deadwood() {
        let config = ClassifierConfig::default();
        let coupling = Coupling::new(0, 1);
        assert_eq!(classify(&coupling, &config), NodeIdentity::IsolatedDeadwood);
    }
}
