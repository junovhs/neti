// src/analysis/v2/inspector.rs
//! Inspection logic for scopes (Metrics application).

use super::metrics;
use super::scope;
use crate::config::RuleConfig;
use crate::types::{Violation, ViolationDetails};

pub struct Inspector<'a> {
    config: &'a RuleConfig,
}

impl<'a> Inspector<'a> {
    #[must_use]
    pub fn new(config: &'a RuleConfig) -> Self {
        Self { config }
    }

    #[must_use]
    pub fn inspect(&self, scope: &scope::Scope) -> Vec<Violation> {
        let mut violations = Vec::new();
        self.check_cohesion(scope, &mut violations);
        self.check_encapsulation(scope, &mut violations);
        self.check_coupling(scope, &mut violations);
        violations
    }

    fn check_cohesion(&self, scope: &scope::Scope, out: &mut Vec<Violation>) {
        if scope.is_enum() || !scope.has_behavior() {
            return;
        }
        if !scope
            .methods()
            .values()
            .any(|m| !m.field_access.is_empty())
        {
            return;
        }
        if scope.methods().len() < 4 {
            return;
        }

        let lcom4 = metrics::ScopeMetrics::calculate_lcom4(scope);
        if lcom4 > self.config.max_lcom4 {
            out.push(build_lcom4_violation(scope, lcom4));
        }
    }

    fn check_encapsulation(&self, scope: &scope::Scope, out: &mut Vec<Violation>) {
        if scope.is_enum() {
            return;
        }
        if is_data_struct(scope) {
            return;
        }
        if is_simple_container(scope) {
            return;
        }
        if !scope.has_behavior() {
            return;
        }
        if scope.fields().is_empty() {
            return;
        }

        let ahf = metrics::ScopeMetrics::calculate_ahf(scope);
        if ahf < self.config.min_ahf {
            out.push(build_ahf_violation(scope, ahf, self.config.min_ahf));
        }
    }

    fn check_coupling(&self, scope: &scope::Scope, out: &mut Vec<Violation>) {
        let cbo = metrics::ScopeMetrics::calculate_cbo(scope);
        let sfout = metrics::ScopeMetrics::calculate_max_sfout(scope);

        if cbo > self.config.max_cbo {
            out.push(build_cbo_violation(scope, cbo));
        }
        if sfout > self.config.max_sfout {
            out.push(build_sfout_violation(scope, sfout));
        }
    }
}

fn is_data_struct(scope: &scope::Scope) -> bool {
    scope.derives().contains("Serialize")
        || scope.derives().contains("Deserialize")
        || scope.derives().contains("Parser")
        || scope.derives().contains("Args")
}

fn is_simple_container(scope: &scope::Scope) -> bool {
    if scope.fields().len() <= 3 {
        return true;
    }
    let total_complexity: usize = scope
        .methods()
        .values()
        .map(|m| m.cognitive_complexity)
        .sum();
    total_complexity <= 3
}

fn build_lcom4_violation(scope: &scope::Scope, lcom4: usize) -> Violation {
    Violation::with_details(
        scope.row(),
        format!(
            "Class '{}' has low cohesion (LCOM4: {})",
            scope.name(),
            lcom4
        ),
        "LCOM4",
        ViolationDetails {
            function_name: Some(scope.name().to_string()),
            analysis: vec![
                format!("Connected components: {lcom4}"),
                format!("Methods: {}", scope.methods().len()),
                "Methods don't share fields or call each other.".into(),
            ],
            suggestion: Some(format!(
                "Split '{}' into {} smaller types.",
                scope.name(),
                lcom4
            )),
        },
    )
}

fn build_ahf_violation(scope: &scope::Scope, ahf: f64, limit: f64) -> Violation {
    let private_count = scope
        .fields()
        .values()
        .filter(|f| !f.is_public)
        .count();
    Violation::with_details(
        scope.row(),
        format!(
            "Class '{}' exposes too much state (AHF: {:.1}%)",
            scope.name(),
            ahf
        ),
        "AHF",
        ViolationDetails {
            function_name: Some(scope.name().to_string()),
            analysis: vec![
                format!("AHF is {ahf:.1}% (min {limit}%)"),
                format!(
                    "{} of {} fields are private.",
                    private_count,
                    scope.fields().len()
                ),
            ],
            suggestion: Some("Encapsulate fields with accessors.".into()),
        },
    )
}

fn build_cbo_violation(scope: &scope::Scope, cbo: usize) -> Violation {
    Violation::with_details(
        scope.row(),
        format!(
            "Class '{}' is tightly coupled (CBO: {})",
            scope.name(),
            cbo
        ),
        "CBO",
        ViolationDetails {
            function_name: Some(scope.name().to_string()),
            analysis: vec![format!("External dependencies: {cbo}")],
            suggestion: Some("Reduce dependencies. Consider facade pattern.".into()),
        },
    )
}

fn build_sfout_violation(scope: &scope::Scope, sfout: usize) -> Violation {
    Violation::with_details(
        scope.row(),
        format!(
            "Class '{}' has high fan-out (SFOUT: {})",
            scope.name(),
            sfout
        ),
        "SFOUT",
        ViolationDetails {
            function_name: Some(scope.name().to_string()),
            analysis: vec![format!("Max outgoing calls: {sfout}")],
            suggestion: Some("Delegate to helper functions.".into()),
        },
    )
}