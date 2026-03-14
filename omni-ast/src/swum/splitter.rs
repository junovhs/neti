//! Identifier splitting for SWUM.

/// Split identifier into words (handles `snake_case` and `camelCase`).
#[must_use]
pub fn split_identifier(name: &str) -> Vec<String> {
    if name.contains('_') {
        return name
            .split('_')
            .filter(|s| !s.is_empty())
            .map(str::to_lowercase)
            .collect();
    }
    split_camel_case(name)
}

/// Split `camelCase`, `PascalCase`, or all-caps identifiers into words.
fn split_camel_case(name: &str) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = name.chars().collect();
    let len = chars.len();

    let mut i = 0;
    while i < len {
        let Some(&ch) = chars.get(i) else { break };
        let next = chars.get(i + 1).copied();
        let prev_lower = i > 0 && chars.get(i - 1).is_some_and(|c| c.is_lowercase());

        if ch.is_uppercase() {
            if prev_lower {
                if !current.is_empty() {
                    words.push(current.to_lowercase());
                    current = String::new();
                }
            } else if let Some(nc) = next {
                if nc.is_lowercase()
                    && !current.is_empty()
                    && current.chars().all(|c| c.is_uppercase())
                {
                    words.push(current.to_lowercase());
                    current = String::new();
                }
            }
        }
        current.push(ch);
        i += 1;
    }

    if !current.is_empty() {
        words.push(current.to_lowercase());
    }

    words
}
