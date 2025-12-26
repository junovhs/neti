# SlopChop v1.0 Brief (Post-Pivot Operating Guide)

**Date:** 2025-12-25  
**Status:** Canonical (living). Supersedes and replaces `/docs/archived/v0.8.0/pivot-brief.md`.  
**Target:** v1.0.0 “Production Ready” trust boundary  
**Current baseline:** v0.9.0 (released; operational / hardening)

---

## 1) Executive Summary

SlopChop has completed the architectural pivot: staged workspace, transactional promotion, hardened parser, and strict patching are in place and operational (v0.9.0).

The remaining work to reach v1.0.0 is not about core safety mechanics; it is about operator trust, coherence, and product finish:

1. **Patch UX & Diagnostics (Phase 3A)**  
   Failures remain safe but are too blunt. Improve “why” and “what to do next” deterministically.

2. **Protocol & Surface Coherence**  
   Standardize PATCH format/semantics and remove legacy delimiter remnants.

3. **CLI Polish & Exit Codes**  
   Predictable automation behavior and stable, documented exit codes.

4. **Machine-Readable Event Log**  
   A stable JSONL stream for auditing and future integrations.

5. **Remove Git-shaped surface area**  
   Eliminate leftover config keys/docs/modules implying Git coupling.

6. **Distribution**  
   Package manager footprints (Homebrew/Scoop/Winget) once interface is stable.

Guiding principle for v1.0.0: **keep strictness, increase legibility**.

---

## 2) Product Thesis and Trust Boundary

### 2.1 What SlopChop is

SlopChop is a repository structural governance gate plus an optional high-integrity change ingestion engine.

It protects a workspace from unsafe/untrusted multi-file payloads while enabling a low-friction “apply/check/promote” loop.

### 2.2 What must always be true (hard invariants)

**Deterministic behavior**
- Same inputs + same workspace state → same outputs and decisions.

**Conservative failure**
- If anything is ambiguous, stale, or malformed: reject. Never “best effort apply” silently.

**Bounded write scope**
- Default writes go only to `.slopchop/stage/worktree`.
- Promotion applies only recorded touched paths to the real workspace.

**Auditable state**
- Every apply/check/promote produces enough structured output (and event log) to explain what happened.

**No Git dependency**
- SlopChop must not require Git presence or a clean Git state to operate safely.

---

## 3) Current State (v0.9.0 Baseline)

### 3.1 Implemented and stable

- Staged workspace: apply writes to `.slopchop/stage/worktree`.
- Transactional promotion: `apply --promote` promotes verified changes to the real workspace, scoped to touched paths with backup/rollback.
- Parser hardening: strict block validation + reserved-name protection.
- Surgical patching: strict search/replace with ambiguity and hash mismatch rejection.
- Tests are green, including patch precision and safety guards.

### 3.2 Known gaps (v1.0.0 blockers)

- Patch failures are safe but blunt: need “Did you mean?” and visual diff summary.
- PATCH format must be canonicalized to the context-anchored form (LEFT_CTX/OLD/RIGHT_CTX/NEW), with the v0.9 SEARCH/REPLACE form retained only as deprecated compatibility (optional).
- Exit codes are not fully standardized/documented.
- Machine-readable event log not yet present.
- Legacy delimiter protocol remnants still exist in at least one code path/test surface.
- Git-shaped config knobs remain (even if functionality is removed).

---

## 4) Goals and Non-Goals for v1.0.0

### 4.1 v1.0.0 Goals (must ship)

1) **Patch UX & Diagnostics**
- Deterministic, bounded, actionable diagnostics for PATCH failures:
  - 0 matches: show closest candidate region (when feasible) and minimal diff summary.
  - >MAX matches: show first N match excerpts and disambiguation guidance.
  - Base hash mismatch: explicit stale-base messaging and next steps.

2) **Protocol Coherence**
- Declare and document the canonical PATCH format and enforce it.
- Ensure unknown/unsupported blocks are rejected and cannot be written as files.
- Remove legacy delimiter protocol from all active flows (and tests that enforce it).

3) **CLI Polish**
- Standardize exit codes across commands (`check`, `apply`, `apply --promote`, `pack`).
- Improve help text and examples so the “one-command loop” is obvious.

4) **Machine-Readable Event Log**
- Emit `.slopchop/events.jsonl` (or a clearly versioned alternative), with stable schema and bounded data.
- Events cover stage lifecycle, apply, write/delete, check, promote, reset.

5) **No Git Surface**
- Remove remaining Git config keys, docs, and dead code paths that imply Git workflows.

### 4.2 Explicit Non-Goals (do not do for v1.0.0)

- Any fuzzy merge, 3-way merge, or heuristic patch application.
- Session IDs or user-facing stage directory management.
- Network/cloud integrations.
- Roadmap/task semantics and “project management” functionality.
- Rich UI/TUI complexity beyond help/diagnostics improvements.

---

## 5) The Operator Loop (Canonical UX)

The intended daily workflow:

1) `slopchop apply`
- Clipboard-first ingestion into stage.

2) `slopchop check`
- Uses stage if present, otherwise workspace.

3) `slopchop apply --promote` (or prompt after green apply/check)
- Promote touched paths from stage to workspace transactionally.

4) `slopchop pack --noprompt`
- Generate context from stage if present; otherwise workspace.

Design rule: the operator should not have to think about stage paths, session identifiers, or internal bookkeeping.

---

## 6) Canonical Protocol Specification (v1.0.0)

### 6.1 XSC7XSC blocks

Supported block kinds in v1.0.0:
- `PLAN` (allowed; does not write)
- `MANIFEST` (parsed safely; never written as a file)
- `FILE` (full file replacement)
- `PATCH` (strict deterministic edit)
- (Future kinds must be rejected unless explicitly implemented)

Rule: unknown kinds are rejected and must never be interpreted as a file path.

### 6.2 Canonical PATCH format (v1.0 contract)

For v1.0, SlopChop will have a single canonical PATCH format: **context-anchored PATCH**.

Canonical format:

XSC7XSC PATCH XSC7XSC path/to/file.rs
BASE_SHA256: <sha256 of current staged file bytes>
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

Semantics (deterministic, byte-exact):
- Read the target file from the staged worktree.
- Reject if `sha256(file_bytes) != BASE_SHA256` (v1.0: **BASE_SHA256 is required**).
- Construct `ANCHOR = LEFT_CTX + OLD + RIGHT_CTX` using literal bytes.
- Find occurrences of `ANCHOR` in the file:
  - If count != MAX_MATCHES: reject (no writes).
  - If count == 1: replace that occurrence of `OLD` with `NEW` inside the matched anchor; contexts must remain unchanged.

Compatibility policy:
- The v0.9 strict SEARCH/REPLACE PATCH form may be accepted as **deprecated compatibility** for a limited period,
  but it must emit a warning and it must not expand the CLI surface.
- All v1.0 UX, diagnostics, and documentation treat the context-anchored format as the standard.


### 6.3 PATCH vs FILE guidance

- If a PATCH payload is “too large” relative to max file budget (e.g., >75%), reject PATCH and request full FILE replacement (or emit a paste-back instruction).
- Must be deterministic and tested.

---

## 7) Phase 3A: Patch UX & Diagnostics (v1.0 Critical)

Phase 3A is implemented against the canonical context-anchored PATCH format; diagnostics for deprecated v0.9 SEARCH/REPLACE are best-effort but bounded and safe.

### 7.1 Diagnostic requirements (always)

- No writes on failure.
- Bounded output: cap excerpts and diff summary sizes deterministically.
- Actionable guidance: always end with a “NEXT:” instruction.

### 7.2 Failure modes and required outputs

**A) Base mismatch**
- Message: PATCH rejected due to stale base (show expected vs actual hash if safe).
- NEXT: run `slopchop pack` (stage-aware) and regenerate patch against current staged file.

**B) 0 matches**
- Message: could not find exact match for PATCH in `<path>`.
- Provide:
  - Probe-based “closest region” (if found) with a bounded excerpt.
  - Minimal diff-like summary between expected and found snippet (bounded).
- NEXT: regenerate PATCH with corrected anchor/context or send full FILE.

**C) >MAX matches**
- Message: PATCH anchor is ambiguous; found N matches.
- Provide:
  - First 2–3 match excerpts with line/byte offsets if available (bounded).
  - Guidance: extend context/anchor to make match unique.
- NEXT: regenerate PATCH with longer context or use FILE.

### 7.3 “Did you mean?” principle (diagnostic locate, not fuzzy apply)

Diagnostics may search for similarity to help correction, but must never apply an approximate match. Suggestions are allowed; application remains exact or rejects.

---

## 8) CLI Polish (v1.0 Requirements)

### 8.1 Exit codes (standardize)

Define and document stable exit codes for:
- Success
- Invalid input / parsing error
- Safety violation (path traversal, reserved names, symlink escape, etc.)
- Patch mismatch / ambiguity / stale base
- Verification failed
- Promotion failed (with rollback result)
- Internal error

Requirement: exit codes must be consistent across platforms and stable for scripting.

### 8.2 Help text and examples

- Ensure `slopchop apply`, `slopchop apply --promote`, `slopchop check`, `slopchop pack --noprompt` explain stage-aware behavior clearly.
- Include at least one example each for FILE payload and PATCH payload.

---

## 9) Machine-Readable Event Log (v1.0 Requirements)

### 9.1 Location and format
- NDJSON / JSONL under `.slopchop/events.jsonl` (preferred), or `.slopchop/stage/events.jsonl` if stage-local.
- Choose one and keep it stable.

### 9.2 Event principles
- Append-only.
- Bounded payload sizes (avoid embedding full file contents).
- Enough metadata to reconstruct what happened:
  - timestamps
  - stage id (if relevant)
  - operation ids
  - paths touched
  - hashes, byte sizes
  - verification command exit codes

### 9.3 Minimum events (v1.0)
- stage_created
- apply_started / apply_rejected / apply_succeeded
- file_written / file_deleted (path, sha256, bytes)
- check_started / check_failed / check_passed
- promote_started / promote_failed / promote_succeeded
- stage_reset

Acceptance criterion: a third-party tool can answer “what changed, where, why, and did it pass checks?”

---

## 10) Surface Area Cleanup (v1.0 Requirements)

### 10.1 Remove Git coupling completely
- Remove Git-related config keys (`auto_commit`, `auto_push`, `commit_prefix`, `allow_dirty_git`) and any docs/help referencing them.
- Delete dead modules/wiring that assume Git.
- Replace “clean working tree” concepts with stage/workspace drift detection only if truly necessary (keep lightweight).

### 10.2 Remove legacy protocol remnants
- Remove the old delimiter protocol from:
  - generation paths
  - tests
  - docs/help
- Ensure only XSC7XSC protocol is recognized and unknown kinds are rejected.

---

## 11) Test Plan and Definition of Done

### 11.1 v1.0.0 tests must cover

1) Parser cannot misinterpret non-FILE blocks as writable paths.  
2) Stage apply writes only under `.slopchop/stage/worktree`.  
3) `check` and `pack` prefer stage when present (explicit tests).  
4) Promotion:
   - touched-path only
   - backups created
   - rollback restores workspace on failure  
5) PATCH:
   - exact match success
   - stale base rejection
   - ambiguity rejection
   - diagnostics for 0 matches and >1 matches  
6) PATCH vs FILE threshold behavior  
7) Event log emission:
   - correct event types
   - stable schema fields present
   - bounded record sizes  

### 11.2 Definition of Done (v1.0.0)

- A user can safely ingest untrusted multi-file payloads into stage, run checks, and promote with deterministic outcomes.
- Patch failures are explainable and actionable without compromising strictness.
- Exit codes are stable and documented.
- Event log exists and is useful.
- No Git-shaped knobs remain.
- No legacy protocol remnants remain.

---

## 12) Release Strategy (Recommended)

### 12.1 v0.9.x (if needed)
Use short-lived patch releases only if you need to stabilize one major area at a time:

- v0.9.1: PATCH diagnostics + diff summaries
- v0.9.2: event log + exit codes
- v0.9.3: legacy/Git surface cleanup

### 12.2 v1.0.0
Ship once:
- PATCH UX is strong
- protocol is coherent
- event log and exit codes are stable
- legacy/Git surfaces are removed

---

## 13) Post-v1.0 (Deliberately Deferred)

Do not mix these into v1.0:
- Optional META block to prevent stale stage patching (only if it materially improves operator outcomes).
- Distribution automation enhancements (CI pipelines for package managers).
- More structured paste-back packet ergonomics.
- Stage history/generations if it proves necessary, but keep non-user-facing.

---

## 14) Immediate Next Actions (Concrete)

**Priority 0 (v1.0 blockers):**
1) Canonicalize PATCH to context-anchored (LEFT_CTX/OLD/RIGHT_CTX/NEW) and update docs/help; optionally keep v0.9 SEARCH/REPLACE as deprecated compatibility.  
2) Implement Phase 3A diagnostics (did-you-mean + bounded visual diff summaries).  
3) Standardize exit codes and document them.  
4) Implement events.jsonl with stable schema and tests.  

**Priority 1 (surface coherence):**
5) Remove Git config keys and any remaining Git-shaped docs/code.  
6) Remove legacy delimiter protocol remnants and tests asserting it.  

**Priority 2 (packaging):**
7) Distribution (Homebrew/Scoop/Winget) once interface is stable.

---
