// src/roadmap/str_utils.rs

#[must_use]
pub fn split_first_word(s: &str) -> (&str, &str) {
    s.trim()
        .split_once(char::is_whitespace)
        .map_or((s.trim(), ""), |(h, t)| (h, t.trim()))
}

/// Parses a quoted string or returns the string as is.
///
/// # Errors
/// Returns error if a starting quote is not closed.
pub fn parse_quoted(s: &str) -> Result<String, String> {
    let (text, _) = extract_quoted_text(s)?;
    Ok(text)
}

/// Parses "text" [AFTER target].
///
/// # Errors
/// Returns error if text quoting is invalid.
pub fn parse_quoted_with_after(s: &str) -> Result<(String, Option<String>), String> {
    let (text, rest) = extract_quoted_text(s)?;

    let after = if let Some(stripped) = rest.strip_prefix("AFTER ") {
        Some(stripped.trim().to_string())
    } else {
        rest.strip_prefix("after ")
            .map(|stripped| stripped.trim().to_string())
    };

    Ok((text, after))
}

/// Extracts quoted text and returns the remainder of the string.
/// Handles escaped quotes (\") inside the string.
///
/// # Errors
/// Returns error if quotes are unbalanced.
pub fn extract_quoted_text(s: &str) -> Result<(String, &str), String> {
    let s = s.trim();
    if let Some(stripped) = s.strip_prefix('"') {
        let end = find_closing_quote(stripped).ok_or("Unclosed quote")?;
        let content = stripped[..end].replace(r#"\""#, "\"");
        Ok((content, stripped[end + 1..].trim()))
    } else if let Some(idx) = s.find(" AFTER ") {
        Ok((s[..idx].trim().to_string(), s[idx..].trim()))
    } else {
        Ok((s.to_string(), ""))
    }
}

fn find_closing_quote(s: &str) -> Option<usize> {
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
        } else if c == '"' {
            return Some(i);
        }
    }
    None
}

#[must_use]
pub fn is_ignorable(line: &str) -> bool {
    let u = line.to_uppercase();
    u.starts_with("===")
        || u.starts_with("---")
        || u.starts_with("```")
        || u.starts_with("#__SLOPCHOP_")
        || u == "ROADMAP"
        || u == "END"
}

#[must_use]
pub fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{truncated}...")
    }
}
