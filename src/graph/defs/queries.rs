use crate::lang::Lang;
use tree_sitter::{Language, Query};

pub struct DefExtractor;

impl DefExtractor {
    #[must_use]
    pub fn get_config(lang: Lang) -> Option<(Language, Query)> {
        let grammar = lang.grammar();
        let query = Query::new(&grammar, lang.q_defs()).ok()?;
        Some((grammar, query))
    }
}
