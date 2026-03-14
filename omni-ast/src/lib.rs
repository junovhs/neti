mod doc_extractor;
mod doc_filter;
pub mod harvester;
pub mod language;
pub mod semantics;
pub mod swum;
pub mod taxonomy;
pub mod types;

pub use harvester::{harvest, SemanticFingerprint};
pub use language::{
    extract_doc_comment_for_file, has_rust_inline_tests, resolve_imports, resolve_primary_symbol,
    resolve_semantic_exports,
};
pub use semantics::{Concept, LangSemantics, SemanticContext, SemanticLanguage};
pub use taxonomy::{load_taxonomy, SemanticBadges, Taxonomy};
pub use types::DepKind;
