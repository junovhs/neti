# SlopChop Pivot Brief (for the next coding AI)
Date: 2025-12-20

This document replaces older planning notes (e.g., `note-on-small-changes.txt`, `chatgpt-ideas.txt`). It captures the current architectural direction and the immediate pivots to implement.

## 0) Context

SlopChop is a repository **structural governance gate** plus an optional **high-integrity change ingestion engine**.

Roadmap has been removed into a separate repository. SlopChop must remain **lean, mean, and repo-focused**.

Key priorities:
- Minimal typing / minimal command surface.
- Deterministic behavior.
- Conservative failure modes.
- Hardened ingestion of untrusted multi-file payloads.
- No coupling to Git workflows (no auto-commit/push; ideally no Git dependency at all).

## 1) Non-goals (explicitly out of scope)

Do not implement these in SlopChop right now:
- Roadmap/task semantics, project management, claim tracking.
- Any “auto git push/commit” features.
- A verbose “session id” CLI UX that requires users to type long commands or paths.
- Entropy/surprisal gating as a replacement for “Three Laws” (may be future audit signal, not a gate).
- Traditional unified diffs / fuzzy merge / 3-way merge. If anything is ambiguous, reject and ask for a clearer patch or a full file.

## 2) UX principle: one-command loop

The user will run (almost always):
- `slopchop apply` (clipboard-first)
- optionally `slopchop apply -c` (apply + verify)
- optionally `slopchop pack --noprompt` (context generation)

Avoid requiring:
- passing session IDs
- passing stage directories
- typing file paths frequently

If additional actions are needed, prefer:
- prompting with `[y/N]`
- emitting a single “NEXT: slopchop apply” instruction
- copying the “paste-back packet” to clipboard automatically

## 3) Chosen architecture: Option B (implicit staged workspace)

We are implementing a **hidden session/sandbox** model without exposing “sessions” to the CLI.

### 3.1 Directory layout (within the real repo)
All SlopChop ephemeral state lives under:

- `.slopchop/`
  - `stage/`
    - `worktree/`        (shadow working tree; a copy of the repo content)
    - `state.json`       (active stage metadata, ids, history pointers)
    - `history/`         (optional generations / archives, retention-limited)
  - `backups/`           (existing apply backups can move here for coherence)

Hard requirements:
- The staged worktree MUST NOT contain a nested `.slopchop/` directory (exclude it during copy).
- The staged worktree MUST be safe to run `slopchop` inside (scans/checks should not see `.slopchop/`).

### 3.2 Behavioral contract

#### `slopchop apply` (default)
- Reads payload from clipboard (or stdin/file as currently supported).
- Ensures stage exists:
  - If `.slopchop/stage/worktree/` does not exist: create it by copying repo → worktree (excluding `.slopchop/`, `.git/`, and other heavy/ignored dirs).
- Applies the payload **into the staged worktree**, not into the real repo.
- If the payload is invalid: reject before writing anything.
- If writing fails: rollback staged worktree changes to its previous state (transactional).
- If apply succeeds:
  - Print a summary.
  - If `-c/--check` (or config default) is enabled: run verification in the staged worktree.
  - If verification is green: prompt to promote staged → real workspace (or auto-promote if configured).
  - If verification fails: generate a paste-back “fix packet” and copy it to clipboard.

#### `slopchop check`
- If a stage exists: run verification in the staged worktree.
- Otherwise: run verification in the real repo.

This keeps the user workflow stable: they can run `slopchop check` and it does the “right thing.”

#### `slopchop pack`
- If a stage exists: pack context from the staged worktree (this is crucial so the AI patches the latest staged submission).
- Otherwise: pack context from the real repo.

#### Promotion (minimal typing)
We want “promotion” without forcing a new command into the user’s muscle memory.

Preferred:
- After a green `apply -c`, prompt: “Promote staged changes to workspace? [y/N]”
- If “y”: perform promotion.
Optional:
- `slopchop apply --promote` to skip the prompt (acceptable; not required).
- A config toggle for “auto promote on green.”

#### Reset / abandon (optional escape hatch)
Keep as a rare escape hatch:
- `slopchop apply --reset` (discard stage)
But avoid adding more commands unless necessary.

## 4) Promotion semantics (staged → real workspace)

Promotion must be safe and deterministic without Git.

Core invariant:
- After promotion, the real workspace should reflect the staged HEAD for all files SlopChop has modified/deleted during the staged session.

Implementation recommendation (lean + controlled):
- Track a set of “touched paths” in `.slopchop/stage/state.json`:
  - union of written/deleted paths across all applies in the current stage
- Promotion applies ONLY those touched paths from stage → repo:
  - For each touched file: copy content from stage/worktree/<path> into real repo/<path>
  - For deletes: delete in real repo
- Promotion must be transactional:
  - backup touched real files first (into `.slopchop/backups/promote/<timestamp>/`)
  - if any write/delete fails: rollback the real workspace to its prior state
- After a successful promote:
  - stage may either remain (for continued work) or be compacted/purged per retention policy (see §6)

Do NOT do “copy the entire worktree over the repo” unless you can guarantee exclusion correctness and no accidental file drift. Manifest/touched-path promotion is safer and leaner.

## 5) Verification in staged worktree (apply + check)

Today the verification pipeline runs external commands and then runs `slopchop` scan.

Pivot requirement:
- Add a `cwd` (current_dir) parameter to verification stages.
- When running in stage mode, all `Command::new(...).args(...).output()` calls must set `.current_dir(stage_worktree)`.

Also ensure:
- the structural scan is run against the staged worktree (running `slopchop` from stage cwd is fine, as long as the scan respects ignores and stage does not include `.slopchop/`).

## 6) “Temporary memory” retention and purge (no Git)

We want a deterministic, Git-independent answer for “when do we clear out temporary state?”

### 6.1 Default policy (simple, safe)
- Keep a single active stage until:
  - user resets it, OR
  - it has been successfully promoted and a new stage is created, OR
  - retention GC runs and prunes old history

### 6.2 History / generations (optional but useful)
If you implement history:
- On each successful `apply`, snapshot stage state metadata (and optionally touched-file copies) into `.slopchop/stage/history/gen_XXXX/`.
- Retain only the last `N` generations (reuse existing `backup_retention` config if you want to avoid adding a new knob).
- History makes debugging and “undo to previous staged state” possible without Git.
- History is NOT a user-facing “session id” UX; it is internal bookkeeping.

### 6.3 Purge triggers (no cloud required)
Keep it local and deterministic:
- After successful promote: prune history to last `N` and optionally clear the active stage if configured.
- On every `apply`: run lightweight GC (delete history entries past retention; delete temp files).

If the user later wants “purge only after cloud upload,” do it via an **optional external command hook**, not Git:
- e.g. `commands.preserve = ["rclone copy ..."]`
- purge only if that command returns success
But this is optional and should not bloat the core.

## 7) Protocol: XSC7XSC, parser hardening, and robust small patches

We are using the Sequence Sigil protocol:

- `XSC7XSC PLAN XSC7XSC ... XSC7XSC END XSC7XSC`
- `XSC7XSC MANIFEST XSC7XSC ... XSC7XSC END XSC7XSC`
- `XSC7XSC FILE XSC7XSC <path> ... XSC7XSC END XSC7XSC`

We will ALSO support a robust “small patch” mechanism that is **not** a unified diff and does **not** do fuzzy application. If anything is ambiguous: reject.

### 7.1 Must-do: harden parsing so future blocks cannot be mis-parsed as files
Current risk: if we add a new block (e.g., `META`) and the extractor treats it as a file path, it could be written to disk.

Required change:
- Parse block kind explicitly and reject unknown kinds, OR
- At minimum, ensure `FILE` block path never equals reserved tokens like `MANIFEST`, `PLAN`, `META`, `PATCH`, empty, etc.

Preferred design (cleanest):
- A single parser that tokenizes blocks into an enum:
  - `Block::Plan(String)`
  - `Block::Manifest(String)`
  - `Block::Meta(String)` (reserved for future)
  - `Block::File { path, content }`
  - `Block::Patch { path, base_sha256, left_ctx, old, right_ctx, new, max_matches }`
- Reject any unknown opener lines.

### 7.2 PATCH blocks (small changes, robust and strict)

PATCH is a deterministic “replace this exact anchored substring” operation.

Format:

XSC7XSC PATCH XSC7XSC path/to/file.rs
BASE_SHA256: <sha256 of the current staged file contents>
MAX_MATCHES: 1
LEFT_CTX:
<literal text>
OLD:
<literal text>
RIGHT_CTX:
<literal text>
NEW:
<literal text>
XSC7XSC END XSC7XSC

Rules:
- Read `path/to/file.rs` from the **staged worktree**.
- Reject if the file’s sha256 does not equal `BASE_SHA256` (stale base; the AI must repack).
- Compute the “anchor” = `LEFT_CTX + OLD + RIGHT_CTX` using literal bytes.
- Find occurrences of `anchor` in the file:
  - If matches != `MAX_MATCHES`, reject the entire apply (no writes).
  - If matches == 1, replace that occurrence of `OLD` with `NEW` and keep contexts unchanged.

PATCH blocks are allowed to be combined with FILE blocks in a single payload, but all paths must still satisfy manifest integrity and path safety rules.

### 7.3 PATCH vs FILE selection rule (automatic guidance)

We keep full-file replacement as the fallback and as the default when the patch is “too big.”

Rule:
- If the PATCH payload for a file (estimate token count of `LEFT_CTX + OLD + RIGHT_CTX + NEW`, or just `NEW` + contexts—choose one and be consistent) exceeds **75% of** `rules.max_file_tokens`, then SlopChop should:
  - reject PATCH for that file and request a `FILE` replacement instead, OR
  - proactively instruct in its paste-back packet: “Send full file for <path>; patch is too large.”

This should be covered by tests and stress tests.

### 7.4 PATCH failure diagnostics: “Did you mean this?” (helpful, not fuzzy-apply)

If a PATCH fails due to:
- 0 matches, OR
- > MAX_MATCHES matches,

SlopChop should print a deterministic diagnostic *without applying anything*:

- “I could not apply PATCH to <path>.”
- If 0 matches:
  - Attempt a **diagnostic locate** (NOT application) to help the AI correct typos:
    - Find a best-effort candidate region by searching for a short probe from LEFT_CTX and RIGHT_CTX (e.g., last 24 chars of LEFT_CTX and first 24 chars of RIGHT_CTX) within the file.
    - If a plausible region is found, print:

      “Found a similar region. Did you mean:”
      - FOUND_LEFT_CTX: <excerpt>
      - FOUND_OLD: <excerpt>
      - FOUND_RIGHT_CTX: <excerpt>

    - Also print a small excerpt of the surrounding file region (bounded, e.g., 200–400 chars).
- If too many matches:
  - Print the first 2–3 matching locations as bounded excerpts to help the AI extend contexts.

This diagnostic is intentionally “fuzzy-adjacent” in that it helps the AI repair its own patch specification, but it never auto-applies an approximate match.

### 7.5 Optional: add a META block for iterative patching (future)
Do not make this required immediately if it slows the pivot.

If added, META exists to support “patch against most recent staged submission”:

XSC7XSC META XSC7XSC
BASE_STAGE_ID: <uuid-or-counter>
PATCH: true
XSC7XSC END XSC7XSC

Rules:
- If stage exists and META.PATCH is true, require BASE_STAGE_ID to match stage state.
- If mismatch: reject and ask for a fresh pack/context.

This prevents the AI from patching stale context.

## 8) Remove Git completely (lean & mean)

We are cutting Git out of SlopChop’s core.

Work items:
- Delete `apply/git` module(s) and any remaining imports/wiring.
- Remove config keys and TUI fields related to:
  - `auto_commit`, `auto_push`, `commit_prefix`, `allow_dirty_git`
- Remove documentation that implies Git operations.

If any “clean working tree” checks remain, replace them with:
- “stage was created from this workspace; workspace changed since stage creation” checks (file fingerprinting or timestamp heuristics).

## 9) Remove/update old-protocol remnants

Search for and eliminate references to the old delimiter protocol (e.g., `#__SLOPCHOP_FILE__#`).

If a feature is still useful (watch mode, TUI ingestion helpers), update it to:
- detect `XSC7XSC ...`
Otherwise, remove the dead code to keep the codebase lean.

## 10) Implementation order (fastest path)

1) Parser hardening (unknown blocks rejected; META/PATCH cannot be mis-parsed as a file).
2) Stage scaffolding:
   - create stage worktree by copying repo (excluding `.slopchop`, `.git`, and heavy dirs).
3) Run verification in a given cwd (stage vs repo).
4) `apply -c` (apply + verify) and ensure `check`/`pack` prefer stage when it exists.
5) Promotion:
   - touched-path tracking + transactional promote.
6) PATCH blocks:
   - implement strict PATCH apply, 75% threshold guidance, and “did you mean this?” diagnostics.
7) Remove Git modules and config keys.
8) Update/remove old-protocol detectors.
9) Emit machine-readable events (see §13).

## 11) Tests (minimum set)

Add/extend tests to cover:
- Parser: `META`/`PATCH` (or any non-FILE block) cannot become a written file path.
- Stage apply writes only under `.slopchop/stage/worktree`.
- `slopchop check` chooses stage when present.
- `slopchop pack` chooses stage when present.
- Promotion reproduces staged content in real workspace for touched files and deletions.
- Rollback works on promote failure (real workspace returns to pre-promote state).
- Stage reset removes stage and state cleanly.
- PATCH success path:
  - exact match replaced correctly, base hash enforced.
- PATCH failure diagnostics:
  - 0 matches produces “did you mean this?” excerpts when possible.
  - >1 matches produces disambiguation excerpts.
- PATCH vs FILE threshold:
  - patches over 75% max token budget are rejected (or cause a requested FILE replacement).
- Stress tests:
  - randomized edits/typos in contexts should reliably reject and produce actionable diagnostics, never silently misapply.

## 12) “Keep it small” guardrails for the coding AI

When implementing:
- Prefer reusing existing writer/backup logic by parameterizing the root directory.
- Avoid introducing a large new CLI surface.
- Avoid “feature creep” (UI polish, complex diff/merge, complicated multi-session management).
- Do not add network/cloud dependencies.
- Keep everything deterministic and testable.

## 13) Machine-readable events (for future Roadmap integration)

SlopChop should emit a machine-readable event stream that Roadmap (or any external tool) can consume later.

Recommendation:
- Write newline-delimited JSON events (JSONL) under `.slopchop/stage/events.jsonl` (or `.slopchop/events.jsonl`).
- Emit events for:
  - stage_created
  - apply_started / apply_rejected / apply_succeeded
  - file_written / file_deleted (path, sha256, bytes)
  - check_started / check_failed / check_passed (commands, exit codes)
  - promote_started / promote_failed / promote_succeeded
  - stage_reset
- Keep the schema stable and small.

---

End of brief.
