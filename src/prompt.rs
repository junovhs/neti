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

CONTEXT STRATEGY (How to drive):

1. IF you receive 'SIGNATURES.txt' (The Map):
   - You are in ARCHITECT MODE.
   - Do NOT write code yet.
   - Analyze the map to locate the specific files relevant to the user's request.
   - INSTRUCT the user to pack those files:
     'Please run: slopchop pack src/foo.rs src/bar.rs --copy'

2. IF you receive 'context.txt' (Source Code):
   - You are in DEVELOPER MODE.
   - You have the implementation details.
   - PROCEED to write the solution using the Output Format below.

{output_format}
"
        )
    }

    fn build_reminder(&self) -> String {
        let tokens = self.config.max_file_tokens;
        let complexity = self.config.max_cyclomatic_complexity;
        let depth = self.config.max_nesting_depth;
        let args = self.config.max_function_args;
        let marker = "#__SLOPCHOP_FILE__#";

        format!(
            r"SLOPCHOP CONSTRAINTS:
□ Files < {tokens} tokens
□ Complexity ≤ {complexity}
□ Nesting ≤ {depth}
□ Args ≤ {args}
□ No .unwrap() or .expect()
□ Use SlopChop Format ({marker} ...)"
        )
    }
}

fn build_output_format() -> String {
    // Note: We construct these strings to avoid the tool parsing THIS file as a command block.
    // We add a space to the file marker in the format string to break the regex match if printed raw.
    let plan = "#__SLOPCHOP_PLAN__#";
    let plan_end = "#__SLOPCHOP_END__#";
    let manifest = "#__SLOPCHOP_MANIFEST__#";
    let manifest_end = "#__SLOPCHOP_END__#";
    let file_header = "#__SLOPCHOP_FILE__#";
    let file_footer = "#__SLOPCHOP_END__#";
    let roadmap = "===ROADMAP===";

    format!(r#"OUTPUT FORMAT (MANDATORY):

1. Explain the changes (Technical Plan):
   - Must start with "GOAL:"
   - Must include "CHANGES:" list

{plan}
GOAL: Refactor authentication module.
CHANGES:
1. Extract user validation to new file.
2. Update config parser.
{plan_end}

2. Declare the plan (Manifest):

{manifest}
path/to/file1.rs
path/to/file2.rs [NEW]
{manifest_end}

3. Provide EACH file:

{file_header} path/to/file1.rs
[file content]
{file_footer}

4. Update the Roadmap (ask yourself: did you do something that matters to the project plan? Record it.):
   - Use this block if you completed a task or need to add one.

{roadmap}
CHECK
id = task-id
ADD
id = new-task
text = Refactor logs
section = v0.2.0
{roadmap}

RULES:
- Do NOT use markdown code blocks (e.g. triple backticks) to wrap the file. The {file_header} delimiters ARE the fence.
- You MAY use markdown inside the file content.
- Every file in the manifest MUST have a matching {file_header} block.
- Paths must match exactly.
- Do NOT truncate files (No "// ...")."#) // slopchop:ignore
}