use crate::semantics::SemanticLanguage;

pub(super) fn collection_access_needles(language: SemanticLanguage) -> &'static [&'static str] {
    match language {
        SemanticLanguage::Rust => &["[0]"],
        SemanticLanguage::Python => &["[0]"],
        SemanticLanguage::JavaScript | SemanticLanguage::TypeScript => &["[0]", ".at(0)"],
        SemanticLanguage::Go => &["[0]"],
        SemanticLanguage::Cpp => &["[0]"],
        SemanticLanguage::Swift => &[],
    }
}

pub(super) fn front_access_needles(language: SemanticLanguage) -> &'static [&'static str] {
    match language {
        SemanticLanguage::Rust => &[".first().unwrap()", ".last().unwrap()"],
        SemanticLanguage::Python => &[],
        SemanticLanguage::JavaScript | SemanticLanguage::TypeScript => &[".shift()"],
        SemanticLanguage::Go => &[],
        SemanticLanguage::Cpp => &[".front()", ".back()"],
        SemanticLanguage::Swift => &[".first!", ".last!"],
    }
}

pub(super) fn collection_guard_needles(language: SemanticLanguage) -> &'static [&'static str] {
    match language {
        SemanticLanguage::Rust => &[".len()", ".is_empty()"],
        SemanticLanguage::Python => &["len(", "if not ", "if "],
        SemanticLanguage::JavaScript | SemanticLanguage::TypeScript => {
            &[".length", ".size", "!== 0", "> 0"]
        }
        SemanticLanguage::Go => &["len("],
        SemanticLanguage::Cpp => &[".size()", ".empty()"],
        SemanticLanguage::Swift => &[".count", ".isEmpty"],
    }
}

pub(super) fn index_identifier_needles(language: SemanticLanguage) -> &'static [&'static str] {
    match language {
        SemanticLanguage::Rust
        | SemanticLanguage::Python
        | SemanticLanguage::JavaScript
        | SemanticLanguage::TypeScript
        | SemanticLanguage::Go
        | SemanticLanguage::Cpp
        | SemanticLanguage::Swift => {
            &["i", "j", "k", "n", "idx", "index", "pos", "ptr", "offset", "cursor"]
        }
    }
}
