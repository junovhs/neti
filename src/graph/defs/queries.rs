// src/graph/defs/queries.rs
use crate::lang::Lang;
use tree_sitter::{Language, Query};

pub struct DefExtractor;

impl DefExtractor {
    #[must_use]
    pub fn get_config(lang: Lang) -> (Language, Query) {
        let grammar = lang.grammar();
        let query = compile_query(grammar, lang.q_defs());
        (grammar, query)
    }
}

fn compile_query(lang: Language, pattern: &str) -> Query {
    Query::new(lang, pattern).unwrap_or_else(|e| panic!("Invalid query: {e}"))
}