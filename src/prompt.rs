// src/prompt.rs
use crate::config::RuleConfig;
use anyhow::Result;

/// Current protocol version for AI compatibility tracking.
const PROTOCOL_VERSION: &str = "1.6.0";

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
    /// Returns error if prompt generation fails.
    pub fn generate(&self) -> Result<String> {
        Ok(build_system_prompt(&self.config))
    }

    /// Generates the concise reminder prompt.
    ///
    /// # Errors
    /// Returns error if reminder generation fails.
    pub fn generate_reminder(&self) -> Result<String> {
        Ok(build_reminder(&self.config))
    }

    /// Wraps the prompt with header/footer.
    ///
    /// # Errors
    /// Returns error if generation fails.
    pub fn wrap_header(&self) -> Result<String> {
        self.generate()
    }

    /// Generates a minimal one-liner for token-constrained contexts.
    #[must_use]
    pub fn generate_short(&self) -> String {
        generate_short_text(&self.config)
    }
}

fn build_system_prompt(config: &RuleConfig) -> String {
    let tokens = config.max_file_tokens;
    // UPDATED: Renamed field
    let complexity = config.max_cognitive_complexity;
    let depth = config.max_nesting_depth;
    let args = config.max_function_args;
    let sigil = "XSC7XSC";

    format!(
        r"SYSTEM MANDATE: THE SLOPCHOP PROTOCOL
ROLE: High-Integrity Systems Architect.
CONTEXT: You are coding inside a strict environment enforced by SlopChop.

THE LAWS:
| Metric | Limit | Catches |
|--------|-------|---------|
| File Tokens | < {tokens} | God files |
| Cognitive Complexity | � {complexity} | Tangled logic |
| Nesting Depth | � {depth} | Deep conditionals |
| Function Args | � {args} | Bloated signatures |
| LCOM4 | = 1 | Incohesive classes (split if > 1) |
| AHF | � 60% | Leaking state (make fields private) |
| CBO | � 9 | Tight coupling (reduce dependencies) |
| SFOUT | � 7 | High fan-out (delegate to helpers) |

LAW OF PARANOIA: No .unwrap() or .expect(). Use Result types.

OUTPUT FORMAT (MANDATORY):
All responses must use the {sigil} DNA sequence sigil. Do NOT use markdown code blocks.

1. Technical Plan:
{sigil} PLAN {sigil}
GOAL: <summary>
CHANGES: <list>
{sigil} END {sigil}

2. Manifest:
{sigil} MANIFEST {sigil}
path/to/file.rs
path/to/new_file.rs [NEW]
{sigil} END {sigil}

3. File Delivery (for new files or major rewrites):
{sigil} FILE {sigil} path/to/file.rs
<raw code content>
{sigil} END {sigil}

4. Surgical Patch (for small, targeted changes to existing files):
{sigil} PATCH {sigil} path/to/file.rs
BASE_SHA256: <sha256 of current staged file bytes>
MAX_MATCHES: 1
LEFT_CTX:
<literal text: code context before OLD>
OLD:
<literal text: the exact code to be replaced>
RIGHT_CTX:
<literal text: code context after OLD>
NEW:
<literal text: the new code to insert>
{sigil} END {sigil}

RULES:
- No truncation. Provide full file contents or complete patch blocks.
- No markdown fences. The {sigil} markers are the fences.
- Use FILE blocks for new files or when changes exceed ~75% of a file.
- Use PATCH blocks for small, targeted changes. Obtain BASE_SHA256 from 'slopchop pack'.
- Run 'slopchop check' after changes. Fix ALL violations before claiming done.
"
    )
}

fn build_reminder(config: &RuleConfig) -> String {
    let sigil = "XSC7XSC";
    format!(
        r"SLOPCHOP v{PROTOCOL_VERSION} CONSTRAINTS:
- Tokens < {}, CC � {}, Depth � {}, Args � {}
- LCOM4 = 1, AHF � 60%, CBO � 9, SFOUT � 7
- Use {sigil} Sigil Protocol (PLAN, MANIFEST, FILE, PATCH)
- Run 'slopchop check' and fix all violations",
        config.max_file_tokens,
        config.max_cognitive_complexity, // UPDATED
        config.max_nesting_depth,
        config.max_function_args,
    )
}

fn generate_short_text(config: &RuleConfig) -> String {
    format!(
        "SlopChop v{}: <{}tok, CC{}, D{}, A{}> Use XSC7XSC protocol.",
        PROTOCOL_VERSION,
        config.max_file_tokens,
        config.max_cognitive_complexity, // UPDATED
        config.max_nesting_depth,
        config.max_function_args,
    )
}