//! Stage 3 semantic badge evaluation.

use crate::harvester::SemanticFingerprint;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[path = "taxonomy_rules.rs"]
mod rules;

pub use rules::Taxonomy;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SemanticBadges {
    pub role: Option<String>,
    pub domain: Option<String>,
    pub mechanisms: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ProjectTaxonomyFile {
    #[serde(default)]
    pub(crate) role: Vec<ProjectRoleRule>,
    #[serde(default)]
    pub(crate) domain: Vec<ProjectDomainRule>,
    #[serde(default)]
    pub(crate) mechanism: Vec<ProjectMechanismRule>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ProjectRoleRule {
    pub(crate) badge: String,
    #[serde(default)]
    pub(crate) exports_contain: Vec<String>,
    #[serde(default)]
    pub(crate) param_type_contains: Vec<String>,
    #[serde(default)]
    pub(crate) annotation_contains: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ProjectDomainRule {
    pub(crate) badge: String,
    pub(crate) string_pattern: String,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ProjectMechanismRule {
    pub(crate) badge: String,
    pub(crate) import_prefix: String,
}

#[must_use]
pub fn load_taxonomy(root: &Path) -> Taxonomy {
    let mut taxonomy = rules::built_in_taxonomy();
    let path = root.join("semmap-taxonomy.yaml");
    let Ok(content) = fs::read_to_string(path) else {
        return taxonomy;
    };
    let Ok(project) = serde_yaml::from_str::<ProjectTaxonomyFile>(&content) else {
        return taxonomy;
    };

    rules::merge_project_rules(&mut taxonomy, project);
    taxonomy
}

impl Taxonomy {
    #[must_use]
    pub fn evaluate(&self, fingerprint: &SemanticFingerprint) -> SemanticBadges {
        rules::evaluate(self, fingerprint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn built_in_taxonomy_detects_mechanism_and_domain() {
        let dir = tempdir().expect("tempdir");
        let taxonomy = load_taxonomy(dir.path());
        let fingerprint = SemanticFingerprint {
            imports: vec![String::from("std::fs"), String::from("database/sql")],
            strings: vec![String::from("https://api.github.com/repos/openai/neti")],
            ..SemanticFingerprint::default()
        };

        let badges = taxonomy.evaluate(&fingerprint);

        assert_eq!(badges.domain.as_deref(), Some("GitHub releases"));
        assert!(badges.mechanisms.contains(&String::from("SQL")));
        assert!(badges.mechanisms.contains(&String::from("file I/O")));
    }

    #[test]
    fn built_in_taxonomy_detects_role_from_exports() {
        let dir = tempdir().expect("tempdir");
        let taxonomy = load_taxonomy(dir.path());
        let fingerprint = SemanticFingerprint {
            exports: vec![String::from("RegisterPlugin")],
            ..SemanticFingerprint::default()
        };

        let badges = taxonomy.evaluate(&fingerprint);

        assert_eq!(badges.role.as_deref(), Some("Registers"));
    }
}
