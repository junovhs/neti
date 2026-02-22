//! Inspection logic for scopes (Metrics application).

use super::scope;
use super::structural;
use crate::config::RuleConfig;
use crate::types::{Confidence, Violation, ViolationDetails};

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
        if !scope.methods().values().any(|m| !m.field_access.is_empty()) {
            return;
        }
        if scope.methods().len() < 4 {
            return;
        }

        let lcom4 = structural::ScopeMetrics::calculate_lcom4(scope);
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

        let ahf = structural::ScopeMetrics::calculate_ahf(scope);
        if ahf < self.config.min_ahf {
            out.push(build_ahf_violation(scope, ahf, self.config.min_ahf));
        }
    }

    fn check_coupling(&self, scope: &scope::Scope, out: &mut Vec<Violation>) {
        let cbo = structural::ScopeMetrics::calculate_cbo(scope);
        let sfout = structural::ScopeMetrics::calculate_max_sfout(scope);

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
    // MEDIUM: accurate measurement but may be expected for trait-heavy types
    let mut v = Violation::with_details(
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
    );
    v.confidence = Confidence::Medium;
    v.confidence_reason =
        Some("accurate metric, but may be expected for types implementing multiple traits".into());
    v
}

fn build_ahf_violation(scope: &scope::Scope, ahf: f64, limit: f64) -> Violation {
    // MEDIUM: libraries intentionally expose fields
    let private_count = scope.fields().values().filter(|f| !f.is_public).count();
    let mut v = Violation::with_details(
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
    );
    v.confidence = Confidence::Medium;
    v.confidence_reason =
        Some("public fields may be intentional API surface (data structs, config)".into());
    v
}

fn build_cbo_violation(scope: &scope::Scope, cbo: usize) -> Violation {
    // MEDIUM: accurate measurement but expected for core/hub types
    let mut v = Violation::with_details(
        scope.row(),
        format!("Class '{}' is tightly coupled (CBO: {})", scope.name(), cbo),
        "CBO",
        ViolationDetails {
            function_name: Some(scope.name().to_string()),
            analysis: vec![format!("External dependencies: {cbo}")],
            suggestion: Some("Reduce dependencies. Consider facade pattern.".into()),
        },
    );
    v.confidence = Confidence::Medium;
    v.confidence_reason =
        Some("accurate measurement, but may be expected for central/hub types".into());
    v
}

fn build_sfout_violation(scope: &scope::Scope, sfout: usize) -> Violation {
    // MEDIUM: high fan-out may be intentional for orchestration functions
    let mut v = Violation::with_details(
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
    );
    v.confidence = Confidence::Medium;
    v.confidence_reason =
        Some("high fan-out may be intentional for orchestrator/coordinator functions".into());
    v
}
