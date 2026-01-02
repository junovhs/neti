# Patch System Stress Testing Plan v1.3

**Status:** Living document  
**Created:** 2025-01-01  
**Purpose:** Systematically verify the patch system won't corrupt files, leak data, or fail silently under adversarial conditions.

---

## Preamble

This code mutates a filesystem based on untrusted input from an LLM. Every edge case is a potential data loss event. Every unchecked path is a security hole. "Works on my machine" is not a testing strategy.

Two critical bugs were found in 30 minutes of ad-hoc testing. This document exists so we don't ship with more.

---

## Test Categories

### 1. Basic Operations

| ID | Test | Expected Behavior | Status |
|----|------|-------------------|--------|
| B01 | Happy path patch | Applies cleanly, file modified correctly | ? PASS |
| B02 | Wrong BASE_SHA256 | Rejected with hash mismatch error | ? PASS |
| B03 | Ambiguous match (OLD appears 2+ times) | Rejected with line numbers shown | ? PASS |
| B04 | Pure insertion (empty OLD) | Content inserted at LEFT_CTX position | ? TODO |
| B05 | Pure deletion (empty NEW) | OLD content removed, nothing inserted | ? TODO |
| B06 | Multiple patches in single payload | Applied sequentially, each sees prior mutations | ? TODO |
| B07 | Patch at byte 0 (no LEFT_CTX) | Matches at file start | ? TODO |
| B08 | Patch at EOF (no RIGHT_CTX) | Matches at file end | ? PASS (implicit) |
| B09 | Patch spans entire file | Equivalent to FILE block replacement | ? TODO |
| B10 | No-op patch (OLD == NEW) | Applies without error, file unchanged | ? TODO |

### 2. Security: Path Traversal & Escape

| ID | Test | Expected Behavior | Status |
|----|------|-------------------|--------|
| S01 | Path `../../../etc/passwd` | Rejected before any write | ? TODO |
| S02 | Path `..\..\..\..\Windows\System32\config` | Rejected (Windows traversal) | ? TODO |
| S03 | Path with embedded null `src/foo\x00bar.rs` | Rejected or sanitized | ? TODO |
| S04 | Absolute path `/etc/passwd` | Rejected | ? TODO |
| S05 | Absolute path `C:\Windows\System32\config` | Rejected (Windows) | ? TODO |
| S06 | Symlink in path pointing outside repo | Rejected with clear error | ? TODO |
| S07 | Symlink created by prior patch in payload | Second patch cannot escape via new symlink | ? TODO |
| S08 | Path with `.` and `..` mixed: `src/./foo/../../../etc/passwd` | Normalized and rejected | ? TODO |
| S09 | Unicode path normalization attack (e.g., NFKC tricks) | Normalized before check | ? TODO |

### 3. Security: Sigil Injection

| ID | Test | Expected Behavior | Status |
|----|------|-------------------|--------|
| I01 | NEW contains END sigil | Parser does not terminate early, content preserved | ? TODO |
| I02 | NEW contains FILE sigil | No new file created, treated as literal text | ? TODO |
| I03 | NEW contains PATCH sigil | No nested patch parsed | ? TODO |
| I04 | LEFT_CTX contains sigil-like text | Matches correctly, no parser confusion | ? TODO |
| I05 | OLD contains sigil-like text | Replaced correctly | ? TODO |
| I06 | Payload with unclosed block | Rejected with clear error | ? TODO |
| I07 | Payload with mismatched block types | Rejected | ? TODO |

### 4. Input Validation: Malformed Patches

| ID | Test | Expected Behavior | Status |
|----|------|-------------------|--------|
| M01 | Missing BASE_SHA256 (V1 format) | Rejected with clear error | ? TODO |
| M02 | Missing OLD section | Rejected | ? TODO |
| M03 | Missing NEW section | Rejected | ? TODO |
| M04 | Sections in wrong order (NEW before OLD) | Rejected or handled gracefully | ? TODO |
| M05 | Duplicate OLD section | Rejected | ? TODO |
| M06 | Truncated payload (cuts off mid-block) | Rejected | ? TODO |
| M07 | Empty patch content (just headers) | Rejected or no-op | ? TODO |
| M08 | BASE_SHA256 wrong length | Rejected | ? TODO |
| M09 | BASE_SHA256 non-hex characters | Rejected | ? TODO |
| M10 | Path in MANIFEST but no FILE/PATCH block | Rejected | ? TODO |
| M11 | FILE/PATCH block not in MANIFEST | Rejected | ? TODO |

### 5. Whitespace & Encoding Gremlins

| ID | Test | Expected Behavior | Status |
|----|------|-------------------|--------|
| W01 | File has LF endings, patch has LF | Matches | ? TODO |
| W02 | File has CRLF endings, patch has LF | Matches (normalized) | ? PASS |
| W03 | File has LF, patch has CRLF | Matches (normalized) | ? TODO |
| W04 | File has mixed line endings | Matches correctly | ? TODO |
| W05 | File has no final newline | Matches | ? PASS |
| W06 | Patch expects trailing newline, file has none | Matches via fallback | ? PASS |
| W07 | Tabs vs spaces in indentation | Exact match required | ? TODO |
| W08 | Trailing whitespace in OLD | Exact match required | ? TODO |
| W09 | Trailing whitespace in file but not patch | Fails (exact match) | ? TODO |
| W10 | Unicode BOM at file start | Handled correctly | ? TODO |
| W11 | UTF-8 content with multi-byte chars | Match boundaries correct | ? TODO |
| W12 | Invalid UTF-8 in file | Rejected or handled gracefully | ? TODO |

### 6. Binary & Hostile Content

| ID | Test | Expected Behavior | Status |
|----|------|-------------------|--------|
| X01 | Null byte in file content | Handled without panic | ? TODO |
| X02 | Null byte in patch content | Handled without panic | ? TODO |
| X03 | Binary file (non-UTF8) | Rejected cleanly | ? TODO |
| X04 | Very large patch (100KB+ content) | Completes without OOM | ? TODO |
| X05 | Very large file (10MB+) | Completes in reasonable time | ? TODO |
| X06 | Single character patch | Works | ? TODO |
| X07 | Empty file target | Patch applies or rejects cleanly | ? TODO |

### 7. Race Conditions & State

| ID | Test | Expected Behavior | Status |
|----|------|-------------------|--------|
| R01 | File deleted between hash check and apply | Clean error, no partial write | ? TODO |
| R02 | File modified between hash check and apply | Hash mismatch (already checked first) | ? PASS (by design) |
| R03 | Stage exists from prior failed apply | New apply creates fresh stage | ? TODO |
| R04 | Promote while stage is being written | Either completes or fails cleanly | ? TODO |
| R05 | Multiple rapid applies | Each gets clean stage | ? TODO |
| R06 | Workspace dirty (uncommitted changes) | Applies cleanly to workspace state | ? TODO |

### 8. Stage & Promotion Integrity

| ID | Test | Expected Behavior | Status |
|----|------|-------------------|--------|
| P01 | Promote success clears stage | Stage dir removed | ? PASS |
| P02 | Promote failure rolls back workspace | Original files restored | ? TODO |
| P03 | Promote with disk full | Clean failure, rollback | ? TODO |
| P04 | Promote with permission denied on target | Clean failure, rollback | ? TODO |
| P05 | Stage contamination after promote | Fresh stage on next apply | ? PASS |
| P06 | Reset clears everything | Stage dir gone, state cleared | ? PASS |
| P07 | Backup retention works | Old backups pruned | ? TODO |

### 9. V0 Format (Deprecated)

| ID | Test | Expected Behavior | Status |
|----|------|-------------------|--------|
| V01 | V0 SEARCH/REPLACE happy path | Works with deprecation warning | ? TODO |
| V02 | V0 without hash | Works (hash optional in V0) | ? TODO |
| V03 | Mixed V0 and V1 in payload | Rejected or each parsed correctly | ? TODO |

---

## Test Execution Protocol

For each test:

1. **Setup:** Clean state (`git checkout . && slopchop apply --reset`)
2. **Execute:** Apply the specific payload
3. **Verify:** Check exact outcome matches expected
4. **Record:** Update status in this document

### Payload Template

Use the standard protocol format with PLAN, MANIFEST, and PATCH/FILE blocks.
See `docs/archived/v1.0.0/features.md` for format reference.

Note: Do not embed live sigil examples in documentation - the parser will try to execute them.

---

## Definition of Done

- All tests in this document marked ? PASS or ? N/A
- Zero silent failures (every error has a clear message)
- Zero data corruption scenarios
- Zero path escape possibilities
- Coverage includes Windows and Unix paths
- Curmudgeonly Rust subreddit contributor has nothing to say

---

## Found Bugs

| Date | Test | Bug | Fix |
|------|------|-----|-----|
| 2025-01-01 | B01/W05 | Trailing newline mismatch caused all patches to fail on CRLF files | `try_match_trimmed` fallback |
| 2025-01-01 | P05 | Stage persisted after promote, contaminating future applies | `self.reset()` after promote |
| 2025-01-01 | I02 | Sigils in FILE content parsed as real blocks | (documented, needs fix) |

---

## Notes

- "Works on my machine" is not in the vocabulary
- Every TODO is technical debt with interest
- This document is updated before PRs are merged
- If you skip a test, document why in the N/A note
- Do not put live sigil markers in documentation files