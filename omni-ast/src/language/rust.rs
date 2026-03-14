//! Rust import extraction and dependency analysis.

use regex::Regex;

pub fn extract_modules(content: &str, crate_name: Option<&str>) -> Vec<String> {
    let mut modules = Vec::new();
    modules.extend(extract_mod_declarations(content));
    modules.extend(extract_crate_imports(content, crate_name));
    modules.sort();
    modules.dedup();
    modules
}

pub fn extract_mod_declarations(content: &str) -> Vec<String> {
    let Ok(re) = Regex::new(r"mod\s+(\w+);") else {
        return Vec::new();
    };
    re.captures_iter(content)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_owned()))
        .collect()
}

pub fn extract_crate_imports(content: &str, crate_name: Option<&str>) -> Vec<String> {
    let mut modules = Vec::new();
    modules.extend(extract_prefixed_imports(content, "crate", true));

    if let Some(name) = crate_name {
        modules.extend(extract_prefixed_imports(content, name, false));
    }

    modules.sort();
    modules.dedup();
    modules
}

pub fn has_inline_tests(content: &str) -> bool {
    let Ok(re) = Regex::new(r"#\s*\[\s*(?:cfg\s*\(\s*test\s*\)|test)\s*\]") else {
        return false;
    };
    re.is_match(content)
}

fn extract_prefixed_imports(
    content: &str,
    prefix: &str,
    include_super_and_self: bool,
) -> Vec<String> {
    let mut modules = Vec::new();
    modules.extend(extract_direct_imports(
        content,
        prefix,
        include_super_and_self,
    ));
    modules.extend(extract_brace_imports(content, prefix));
    modules
}

fn extract_direct_imports(
    content: &str,
    prefix: &str,
    include_super_and_self: bool,
) -> Vec<String> {
    let pattern = if include_super_and_self {
        format!(r"use\s+(?:{prefix}|super|self)::(\w+)")
    } else {
        format!(r"use\s+{prefix}::(\w+)")
    };
    let Ok(re) = Regex::new(&pattern) else {
        return Vec::new();
    };
    re.captures_iter(content)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_owned()))
        .collect()
}

fn extract_brace_imports(content: &str, prefix: &str) -> Vec<String> {
    let pattern = format!(r"use\s+{prefix}::\{{([^}}]+)\}}");
    let Ok(re) = Regex::new(&pattern) else {
        return Vec::new();
    };
    let mut modules = Vec::new();
    for cap in re.captures_iter(content) {
        if let Some(m) = cap.get(1) {
            parse_brace_group(m.as_str(), &mut modules);
        }
    }
    modules
}

fn parse_brace_group(group: &str, modules: &mut Vec<String>) {
    let found: Vec<String> = group
        .split(',')
        .filter_map(|item| {
            let segment = item.trim().split("::").next()?.trim();
            if is_module_name(segment) {
                Some(segment.to_owned())
            } else {
                None
            }
        })
        .collect();
    modules.extend(found);
}

fn is_module_name(s: &str) -> bool {
    !s.is_empty() && s.chars().next().is_some_and(|c| c.is_lowercase())
}
