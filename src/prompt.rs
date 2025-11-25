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
    ///
    /// # Errors
    /// This function does not currently error but returns Result for API consistency.
    pub fn generate(&self) -> Result<String> {
        Ok(self.build_system_prompt())
    }

    /// Wraps the header for context packs.
    ///
    /// # Errors
    /// This function does not currently error but returns Result for API consistency.
    pub fn wrap_header(&self) -> Result<String> {
        Ok(self.build_system_prompt())
    }

    /// Generates a short reminder prompt.
    ///
    /// # Errors
    /// This function does not currently error but returns Result for API consistency.
    pub fn generate_reminder(&self) -> Result<String> {
        Ok(self.build_reminder())
    }

    fn build_system_prompt(&self) -> String {
        let tokens = self.config.max_file_tokens;
        let complexity = self.config.max_cyclomatic_complexity;
        let depth = self.config.max_nesting_depth;
        let args = self.config.max_function_args;
        let output_format = build_output_format();

        format!(
            r"🛡️ SYSTEM MANDATE: THE WARDEN PROTOCOL
ROLE: High-Integrity Systems Architect (NASA/JPL Standard).
CONTEXT: You are coding inside a strict environment enforced by Warden.

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
            r"WARDEN CONSTRAINTS:
□ Files < {tokens} tokens
□ Complexity ≤ {complexity}
□ Nesting ≤ {depth}
□ Args ≤ {args}
□ No .unwrap()
□ Use <delivery> + <file> tags"
        )
    }
}

fn build_output_format() -> &'static str {
    r"OUTPUT FORMAT (MANDATORY):

When providing code files, use this exact format:

1. Declare ALL files first:

<delivery>
path/to/file1.rs
path/to/file2.rs [NEW]
</delivery>

2. Provide EACH file:

<file path='path/to/file1.rs'>
[complete file contents]
</file>

RULES:
- Every file in <delivery> MUST have a matching <file> block
- Do NOT use markdown code blocks - use <file> tags only
- Do NOT truncate files
- Paths must match exactly

The `warden apply` command will REJECT incomplete deliveries."
}
