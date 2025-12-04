// src/prompt.rs
use crate::config::RuleConfig;
use anyhow::Result;

pub struct PromptGenerator {
    config: RuleConfig,
}

impl PromptGenerator {
    #[must_use]
    pub fn new(config: RuleConfig) -> Self {
        Self { config }
    }

    /// Generates the full system prompt.
    /// # Errors
    /// Currently infallible, returns Result for API consistency.
    pub fn generate(&self) -> Result<String> {
        Ok(self.build_system_prompt())
    }

    /// Generates a short reminder prompt for context footers.
    /// # Errors
    /// Currently infallible, returns Result for API consistency.
    pub fn generate_reminder(&self) -> Result<String> {
        Ok(self.build_reminder())
    }

    /// Alias for `generate()` — used by knit for context headers.
    /// # Errors
    /// Currently infallible, returns Result for API consistency.
    pub fn wrap_header(&self) -> Result<String> {
        self.generate()
    }

    fn build_system_prompt(&self) -> String {
        let tokens = self.config.max_file_tokens;
        let complexity = self.config.max_cyclomatic_complexity;
        let depth = self.config.max_nesting_depth;
        let args = self.config.max_function_args;
        let output_format = build_output_format();

        format!(
            r"🛡️ SYSTEM MANDATE: THE SLOPCHOP PROTOCOL
ROLE: High-Integrity Systems Architect (NASA/JPL Standard).
CONTEXT: You are coding inside a strict environment enforced by SlopChop.

THE 3 LAWS (Non-Negotiable):

1. LAW OF ATOMICITY
   - Files: MUST be < {tokens} tokens.
   - Action: Split immediately if larger.

2. LAW OF COMPLEXITY
   - Cyclomatic Complexity: MUST be ≤ {complexity} per function.
   - Nesting Depth: MUST be ≤ {depth} levels.
   - Function Arguments: MUST be ≤ {args} parameters.

3. LAW OF PARANOIA
   - Use Result<T, E> for I/O and fallible operations.
   - NO .unwrap() or .expect() calls.

{output_format}
"
        )
    }

    fn build_reminder(&self) -> String {
        let tokens = self.config.max_file_tokens;
        let complexity = self.config.max_cyclomatic_complexity;
        let depth = self.config.max_nesting_depth;
        let args = self.config.max_function_args;

        format!(
            r"SLOPCHOP CONSTRAINTS:
□ Files < {tokens} tokens
□ Complexity ≤ {complexity}
□ Nesting ≤ {depth}
□ Args ≤ {args}
□ No .unwrap() or .expect()
□ Use SlopChop Format (#__SLOPCHOP_FILE__# ... #__SLOPCHOP_END__#)"
        )
    }
}

fn build_output_format() -> String {
    r#"OUTPUT FORMAT (MANDATORY):

1. Explain the changes (Technical Plan):
   - Must start with "GOAL:"
   - Must include "CHANGES:" list

#__SLOPCHOP_PLAN__#
GOAL: Refactor authentication module.
CHANGES:
1. Extract user validation to new file.
2. Update config parser.
#__SLOPCHOP_END__#

2. Declare the plan (Manifest):

#__SLOPCHOP_MANIFEST__#
path/to/file1.rs
path/to/file2.rs [NEW]
#__SLOPCHOP_END__#

3. Provide EACH file:

#__SLOPCHOP_FILE__# path/to/file1.rs
[file content]
#__SLOPCHOP_END__#

RULES:
- Do NOT use markdown code blocks (e.g. triple backticks) to wrap the file. The #__SLOPCHOP_FILE__# delimiters ARE the fence.
- You MAY use markdown inside the file content.
- Every file in the manifest MUST have a matching #__SLOPCHOP_FILE__# block.
- Paths must match exactly.
- Do NOT truncate files (No "// ...")."#.to_string()
}
