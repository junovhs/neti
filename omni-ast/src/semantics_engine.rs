use super::{Concept, LangSemantics, SemanticContext, SemanticLanguage};
use super::{concurrency_queries, logic_queries, queries};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SharedSemantics { // neti:allow(CBO) // neti:allow(SFOUT) Central semantic hub by design.
    language: SemanticLanguage,
}

#[must_use]
pub fn semantics_for(language: SemanticLanguage) -> SharedSemantics {
    SharedSemantics { language }
}

impl SemanticContext {
    #[must_use]
    pub fn from_source(source_text: impl Into<String>) -> Self {
        Self {
            source_text: source_text.into(),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn with_path(mut self, path: impl AsRef<Path>) -> Self {
        self.path_hint = Some(path.as_ref().to_string_lossy().into_owned());
        self
    }
}

impl LangSemantics for SharedSemantics {
    fn language(&self) -> SemanticLanguage {
        self.language
    }

    fn is_test_context(&self, context: &SemanticContext) -> bool {
        queries::is_test_context(self.language, context)
    }

    fn has_concept(&self, concept: Concept, context: &SemanticContext) -> bool {
        queries::has_concept(self.language, concept, context)
    }

    fn has_length_boundary_risk(&self, context: &SemanticContext) -> bool {
        logic_queries::has_length_boundary_risk(self.language, context)
    }

    fn has_unguarded_collection_access(&self, context: &SemanticContext) -> bool {
        logic_queries::has_unguarded_collection_access(self.language, context)
    }

    fn has_unwrapped_front_access(&self, context: &SemanticContext) -> bool {
        logic_queries::has_unwrapped_front_access(self.language, context)
    }

    fn has_guarding_collection_check(&self, context: &SemanticContext) -> bool {
        logic_queries::has_guarding_collection_check(self.language, context)
    }

    fn is_async_locking_context(&self, context: &SemanticContext) -> bool {
        concurrency_queries::is_async_locking_context(self.language, context)
    }
}
