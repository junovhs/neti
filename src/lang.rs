// src/lang.rs
use tree_sitter::Language;

#[path = "lang_queries.rs"]
mod lang_queries;
use lang_queries::QUERIES;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    Rust,
    Python,
    TypeScript,
    Swift,
}

#[derive(Debug, Clone, Copy)]
pub enum QueryKind {
    Naming,
    Complexity,
    Imports,
    Defs,
    Exports,
    Skeleton,
}

impl Lang {
    #[must_use]
    pub fn from_ext(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Self::Rust),
            "py" => Some(Self::Python),
            "ts" | "tsx" | "js" | "jsx" => Some(Self::TypeScript),
            "swift" => Some(Self::Swift),
            _ => None,
        }
    }

    #[must_use]
    pub fn grammar(self) -> Language {
        match self {
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Swift => tree_sitter_swift::LANGUAGE.into(),
        }
    }

    #[must_use]
    #[allow(clippy::indexing_slicing)]
    pub fn query(self, kind: QueryKind) -> &'static str {
        let lang_idx = self as usize;
        let query_idx = kind as usize;
        QUERIES[lang_idx][query_idx]
    }

    #[must_use]
    pub fn q_naming(self) -> &'static str {
        self.query(QueryKind::Naming)
    }
    #[must_use]
    pub fn q_complexity(self) -> &'static str {
        self.query(QueryKind::Complexity)
    }
    #[must_use]
    pub fn q_imports(self) -> &'static str {
        self.query(QueryKind::Imports)
    }
    #[must_use]
    pub fn q_defs(self) -> &'static str {
        self.query(QueryKind::Defs)
    }
    #[must_use]
    pub fn q_exports(self) -> &'static str {
        self.query(QueryKind::Exports)
    }
    #[must_use]
    pub fn q_skeleton(self) -> &'static str {
        self.query(QueryKind::Skeleton)
    }

    #[must_use]
    pub fn skeleton_replacement(self) -> &'static str {
        if self == Self::Python {
            "\n    ..."
        } else {
            "{ ... }"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_from_ext() {
        assert_eq!(Lang::from_ext("rs"), Some(Lang::Rust));
        assert_eq!(Lang::from_ext("swift"), Some(Lang::Swift));
        assert_eq!(Lang::from_ext("py"), Some(Lang::Python));
        assert_eq!(Lang::from_ext("ts"), Some(Lang::TypeScript));
        assert_eq!(Lang::from_ext("xyz"), None);
    }

    #[test]
    fn test_swift_queries_compile() {
        let lang: Language = tree_sitter_swift::LANGUAGE.into();
        validate_swift_query(&lang, QueryKind::Naming);
        validate_swift_query(&lang, QueryKind::Complexity);
        validate_swift_query(&lang, QueryKind::Imports);
        validate_swift_query(&lang, QueryKind::Defs);
        validate_swift_query(&lang, QueryKind::Exports);
        validate_swift_query(&lang, QueryKind::Skeleton);
    }

    fn validate_swift_query(lang: &Language, kind: QueryKind) {
        // neti:allow(P03)
        let q = Lang::Swift.query(kind);
        let result = tree_sitter::Query::new(lang, q);
        assert!(
            result.is_ok(),
            "Swift query failed for {kind:?}: {:?}",
            result.err()
        );
    }
}
