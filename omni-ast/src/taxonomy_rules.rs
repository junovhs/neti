use crate::harvester::SemanticFingerprint;
use crate::taxonomy::{
    ProjectDomainRule, ProjectMechanismRule, ProjectRoleRule, ProjectTaxonomyFile, SemanticBadges,
};
use regex::Regex;

#[derive(Debug, Clone, Default)]
pub struct Taxonomy {
    pub(crate) role: Vec<RoleRule>,
    pub(crate) domain: Vec<DomainRule>,
    pub(crate) mechanism: Vec<MechanismRule>,
}

#[derive(Debug, Clone)]
pub(crate) struct RoleRule {
    pub(crate) badge: String,
    pub(crate) exports_contain: Vec<String>,
    pub(crate) param_type_contains: Vec<String>,
    pub(crate) annotation_contains: Vec<String>,
    pub(crate) score: u8,
    pub(crate) order: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct DomainRule {
    pub(crate) badge: String,
    pub(crate) string_pattern: String,
    pub(crate) score: u8,
    pub(crate) order: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct MechanismRule {
    pub(crate) badge: String,
    pub(crate) import_prefix: String,
    pub(crate) score: u8,
    pub(crate) order: usize,
}

pub(crate) fn built_in_taxonomy() -> Taxonomy {
    Taxonomy {
        role: vec![
            role_rule(0, "Manages", &["Install", "Uninstall"], &[], &[]),
            role_rule(1, "Fetches", &["Fetch"], &[], &[]),
            role_rule(
                2,
                "Handles HTTP requests for",
                &[],
                &["http.ResponseWriter"],
                &[],
            ),
            role_rule(3, "Persists", &[], &[], &["gorm", "db", "serde"]),
            role_rule(4, "Registers", &["Register"], &[], &[]),
        ],
        domain: vec![
            domain_rule(5, "GitHub releases", r"api\.github\.com|/repos/"),
            domain_rule(6, "macOS launch agents", r"Library/LaunchAgents"),
            domain_rule(7, "Windows registry autostart", r"CurrentVersion\\Run"),
        ],
        mechanism: vec![
            mechanism_rule(8, "HTTP API", "net/http"),
            mechanism_rule(9, "SQL", "database/sql"),
            mechanism_rule(10, "Windows registry", "golang.org/x/sys/windows/registry"),
            mechanism_rule(11, "subprocess execution", "os/exec"),
            mechanism_rule(12, "JSON serialization", "encoding/json"),
            mechanism_rule(13, "file I/O", "std::fs"),
        ],
    }
}

pub(crate) fn merge_project_rules(taxonomy: &mut Taxonomy, project: ProjectTaxonomyFile) {
    let mut next_order = taxonomy.role.len() + taxonomy.domain.len() + taxonomy.mechanism.len();
    for rule in project.role {
        taxonomy.role.insert(0, project_role_rule(rule, next_order));
        next_order += 1;
    }
    for rule in project.domain {
        if Regex::new(&rule.string_pattern).is_ok() {
            taxonomy
                .domain
                .insert(0, project_domain_rule(rule, next_order));
            next_order += 1;
        }
    }
    for rule in project.mechanism {
        taxonomy
            .mechanism
            .insert(0, project_mechanism_rule(rule, next_order));
        next_order += 1;
    }
}

pub(crate) fn evaluate(taxonomy: &Taxonomy, fingerprint: &SemanticFingerprint) -> SemanticBadges {
    SemanticBadges {
        role: select_role(&taxonomy.role, fingerprint),
        domain: select_domain(&taxonomy.domain, fingerprint),
        mechanisms: select_mechanisms(&taxonomy.mechanism, fingerprint),
    }
}

fn role_rule(
    order: usize,
    badge: &str,
    exports_contain: &[&str],
    param_type_contains: &[&str],
    annotation_contains: &[&str],
) -> RoleRule {
    let score = (!exports_contain.is_empty() as u8)
        + ((!param_type_contains.is_empty() as u8) * 2)
        + ((!annotation_contains.is_empty() as u8) * 3);
    RoleRule {
        badge: badge.to_string(),
        exports_contain: exports_contain.iter().map(|s| (*s).to_string()).collect(),
        param_type_contains: param_type_contains
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
        annotation_contains: annotation_contains
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
        score,
        order,
    }
}

fn domain_rule(order: usize, badge: &str, pattern: &str) -> DomainRule {
    DomainRule {
        badge: badge.to_string(),
        string_pattern: pattern.to_string(),
        score: 3,
        order,
    }
}

fn mechanism_rule(order: usize, badge: &str, import_prefix: &str) -> MechanismRule {
    MechanismRule {
        badge: badge.to_string(),
        import_prefix: import_prefix.to_string(),
        score: 1,
        order,
    }
}

fn project_role_rule(rule: ProjectRoleRule, order: usize) -> RoleRule {
    let score = (!rule.exports_contain.is_empty() as u8)
        + ((!rule.param_type_contains.is_empty() as u8) * 2)
        + ((!rule.annotation_contains.is_empty() as u8) * 3);
    RoleRule {
        badge: rule.badge,
        exports_contain: rule.exports_contain,
        param_type_contains: rule.param_type_contains,
        annotation_contains: rule.annotation_contains,
        score,
        order,
    }
}

fn project_domain_rule(rule: ProjectDomainRule, order: usize) -> DomainRule {
    DomainRule {
        badge: rule.badge,
        string_pattern: rule.string_pattern,
        score: 3,
        order,
    }
}

fn project_mechanism_rule(rule: ProjectMechanismRule, order: usize) -> MechanismRule {
    MechanismRule {
        badge: rule.badge,
        import_prefix: rule.import_prefix,
        score: 1,
        order,
    }
}

fn select_role(rules: &[RoleRule], fingerprint: &SemanticFingerprint) -> Option<String> {
    let mut best: Option<(u8, usize, String)> = None;
    for rule in rules {
        if role_matches(rule, fingerprint) {
            let candidate = (rule.score, usize::MAX - rule.order, rule.badge.clone());
            if best.as_ref().is_none_or(|current| candidate > *current) {
                best = Some(candidate);
            }
        }
    }
    best.map(|(_, _, badge)| badge)
}

fn role_matches(rule: &RoleRule, fingerprint: &SemanticFingerprint) -> bool {
    let exports_match = rule.exports_contain.is_empty()
        || rule.exports_contain.iter().all(|needle| {
            fingerprint
                .exports
                .iter()
                .any(|export| export.contains(needle))
        });
    let params_match = rule.param_type_contains.is_empty()
        || rule.param_type_contains.iter().any(|needle| {
            fingerprint
                .param_types
                .iter()
                .any(|param| param.contains(needle))
        });
    let annotation_match = rule.annotation_contains.is_empty()
        || rule.annotation_contains.iter().any(|needle| {
            fingerprint
                .annotations
                .iter()
                .any(|ann| ann.contains(needle))
        });
    exports_match && params_match && annotation_match
}

fn select_domain(rules: &[DomainRule], fingerprint: &SemanticFingerprint) -> Option<String> {
    let mut best: Option<(u8, usize, String)> = None;
    for rule in rules {
        if fingerprint.strings.iter().any(|value| {
            Regex::new(&rule.string_pattern)
                .ok()
                .is_some_and(|re| re.is_match(value))
        }) {
            let candidate = (rule.score, usize::MAX - rule.order, rule.badge.clone());
            if best.as_ref().is_none_or(|current| candidate > *current) {
                best = Some(candidate);
            }
        }
    }
    best.map(|(_, _, badge)| badge)
}

fn select_mechanisms(rules: &[MechanismRule], fingerprint: &SemanticFingerprint) -> Vec<String> {
    let mut matches: Vec<(u8, usize, String)> = rules
        .iter()
        .filter(|rule| {
            fingerprint
                .imports
                .iter()
                .any(|import| import.starts_with(&rule.import_prefix))
        })
        .map(|rule| (rule.score, usize::MAX - rule.order, rule.badge.clone()))
        .collect();
    matches.sort_by(|a, b| b.cmp(a));
    matches
        .into_iter()
        .map(|(_, _, badge)| badge)
        .fold(Vec::new(), |mut acc, badge| {
            if !acc.contains(&badge) && acc.len() < 2 {
                acc.push(badge);
            }
            acc
        })
}
