use super::logic_tables::{
    collection_access_needles, collection_guard_needles, front_access_needles,
    index_identifier_needles,
};
use super::queries::has_concept;
use super::{Concept, SemanticContext, SemanticLanguage};

pub(super) fn has_length_boundary_risk(
    language: SemanticLanguage,
    context: &SemanticContext,
) -> bool {
    let source = context.source_text.trim();
    let Some((left, op, right)) = split_comparison(source) else {
        return false;
    };

    let left_is_length = has_concept(
        language,
        Concept::Length,
        &SemanticContext::from_source(left.to_owned()),
    );
    let right_is_length = has_concept(
        language,
        Concept::Length,
        &SemanticContext::from_source(right.to_owned()),
    );

    if left_is_length == right_is_length {
        return false;
    }

    if right_is_length && op == "<=" {
        return operand_can_be_index(language, left);
    }

    if left_is_length && op == ">=" {
        return operand_can_be_index(language, right);
    }

    false
}

pub(super) fn has_unguarded_collection_access(
    language: SemanticLanguage,
    context: &SemanticContext,
) -> bool {
    contains_any(context.source_text.as_str(), collection_access_needles(language))
}

pub(super) fn has_unwrapped_front_access(
    language: SemanticLanguage,
    context: &SemanticContext,
) -> bool {
    contains_any(context.source_text.as_str(), front_access_needles(language))
}

pub(super) fn has_guarding_collection_check(
    language: SemanticLanguage,
    context: &SemanticContext,
) -> bool {
    contains_any(context.source_text.as_str(), collection_guard_needles(language))
}

fn split_comparison(source: &str) -> Option<(&str, &str, &str)> {
    if let Some((left, right)) = source.split_once("<=") {
        return Some((left.trim(), "<=", right.trim()));
    }
    if let Some((left, right)) = source.split_once(">=") {
        return Some((left.trim(), ">=", right.trim()));
    }
    None
}

fn operand_can_be_index(language: SemanticLanguage, operand: &str) -> bool {
    let normalized = normalize_operand(operand);
    if normalized.is_empty() || normalized.chars().all(|ch| ch.is_ascii_digit()) {
        return false;
    }

    let lowered = normalized.to_lowercase();
    if index_identifier_needles(language)
        .iter()
        .any(|needle| lowered == *needle || lowered.contains(needle))
    {
        return true;
    }

    lowered
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '[' | ']'))
        && !lowered.contains("len")
        && !lowered.contains("size")
        && !lowered.contains("empty")
}

fn normalize_operand(operand: &str) -> &str {
    operand
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim_end_matches(':')
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    let haystack = haystack.to_lowercase();
    needles
        .iter()
        .any(|needle| haystack.contains(&needle.to_lowercase()))
}
