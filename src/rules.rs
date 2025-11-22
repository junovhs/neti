use crate::error::Result;
use colored::Colorize;
use regex::Regex;
use std::fs;
use std::path::Path;

pub struct RuleEngine {
    fn_rust: Regex,
    fn_py: Regex,
    fn_ts: Regex,
    fn_sh: Regex,
    class_css: Regex,
}

impl RuleEngine {
    /// Creates a new rule engine.
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded regex patterns are invalid (test-covered).
    #[must_use]
    pub fn new() -> Self {
        Self {
            fn_rust: Regex::new(r"fn\s+([a-z0-9_]+)\s*\(").expect("Bad Regex"),
            fn_py: Regex::new(r"def\s+([a-z0-9_]+)\s*\(").expect("Bad Regex"),
            fn_ts: Regex::new(r"(?:function\s+([a-zA-Z0-9]+)\s*\(|const\s+([a-zA-Z0-9]+)\s*=\s*(?:async\s*)?(?:function|\())").expect("Bad Regex"),
            fn_sh: Regex::new(r"^([a-z0-9_]+)\(\)\s*\{").expect("Bad Regex"),
            class_css: Regex::new(r"\.([a-z0-9_-]+)").expect("Bad Regex"),
        }
    }

    /// Checks a file for rule violations.
    ///
    /// # Errors
    ///
    /// Returns error if file reading fails (though logic generally suppresses it).
    pub fn check_file(&self, path: &Path) -> Result<bool> {
        let Ok(content) = fs::read_to_string(path) else {
            return Ok(true); // Skip binary/unreadable gracefully
        };

        let mut passed = true;
        let filename = path.to_string_lossy();

        let line_count = content.lines().count();
        if line_count > 200 {
            println!(
                "{} {}: {} lines (Limit: 200). Split this file.",
                "[BLOAT]".red().bold(),
                filename,
                line_count
            );
            passed = false;
        }

        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            match ext {
                "rs" => {
                    Self::check_names(&content, &self.fn_rust, "_", 3, path, &mut passed);
                }
                "py" => {
                    Self::check_names(&content, &self.fn_py, "_", 3, path, &mut passed);
                }
                "ts" | "js" | "tsx" | "jsx" => {
                    Self::check_names_camel(&content, &self.fn_ts, 3, path, &mut passed);
                }
                "sh" => {
                    Self::check_names(&content, &self.fn_sh, "_", 3, path, &mut passed);
                }
                "css" => {
                    Self::check_names(&content, &self.class_css, "-", 3, path, &mut passed);
                }
                _ => {}
            }
        }

        let is_code = matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go")
        );

        if is_code {
            let lower = content.to_lowercase();
            let has_safety = lower.contains("result")
                || lower.contains("option")
                || lower.contains("try")
                || lower.contains("catch")
                || lower.contains("except")
                || lower.contains("unwrap_or");

            if !has_safety {
                println!(
                    "{} {}: No obvious error handling found (Result, try/catch, etc).",
                    "[UNSAFE]".yellow().bold(),
                    filename
                );
            }
        }

        Ok(passed)
    }

    fn check_names(
        content: &str,
        regex: &Regex,
        separator: &str,
        limit: usize,
        path: &Path,
        passed: &mut bool,
    ) {
        for cap in regex.captures_iter(content) {
            if let Some(name_match) = cap.get(1) {
                let name = name_match.as_str();
                let parts = name.split(separator).count();
                if parts > limit {
                    println!(
                        "{} {}: Function '{}' has {} words (Limit: 3).",
                        "[VERBOSE]".red().bold(),
                        path.to_string_lossy(),
                        name,
                        parts
                    );
                    *passed = false;
                }
            }
        }
    }

    fn check_names_camel(
        content: &str,
        regex: &Regex,
        limit: usize,
        path: &Path,
        passed: &mut bool,
    ) {
        for cap in regex.captures_iter(content) {
            let name = cap.get(1).or_else(|| cap.get(2)).map_or("", |m| m.as_str());

            let caps_count = name.chars().filter(|c| c.is_uppercase()).count();
            if 1 + caps_count > limit {
                println!(
                    "{} {}: Function '{}' is too complex (Limit: 3 words).",
                    "[VERBOSE]".red().bold(),
                    path.to_string_lossy(),
                    name
                );
                *passed = false;
            }
        }
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}
