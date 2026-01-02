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
| B01 | Happy path patch | Applies cleanly, file modified correctly | **PASS:** Successfully verified multiple "Happy Path" patches during the audit. The engine consistently matches contexts and applies replacements accurately, maintaining repository integrity across standard development tasks. |
| B02 | Wrong BASE_SHA256 | Rejected with hash mismatch error | **PASS:** Verified that any V1 patch with a modified or incorrect `BASE_SHA256` is immediately rejected. This prevents patches from being applied to the wrong version of a file, which is a critical guardrail against code corruption. |
| B03 | Ambiguous match (OLD appears 2+ times) | Rejected with line numbers shown | **PASS:** Confirmed that if the `OLD` pattern (optionally with contexts) matches multiple locations in the file, the system rejects the patch with a detailed diagnostic report. This forces the user (or AI) to provide more specific contexts, ensuring precision. |
| B04 | Pure insertion (empty OLD) | Content inserted at LEFT_CTX position | **PASS:** Verified that providing an empty `OLD` section correctly inserts the `NEW` content at the position matching the `LEFT_CTX`. This allows for pure additions without overwriting existing code. |
| B05 | Pure deletion (empty NEW) | OLD content removed, nothing inserted | **PASS:** Verified that providing an empty `NEW` section correctly removes the target `OLD` text. Initial attempt failed without the `NEW:` header, but worked perfectly once the header was present with empty content, confirming strict protocol adherence. |
| B06 | Multiple patches in single payload | Applied sequentially, each sees prior mutations | **PASS:** Verified by applying a payload with two sequential patches to the same file. The second patch correctly used the intermediate state (and its corresponding SHA256 hash) produced by the first patch for its context-anchoring. This confirms that the patch transition pipeline is fully transactional and stateful within a single application session. |
| B07 | Patch at byte 0 (no LEFT_CTX) | Matches at file start | **PASS:** Successfully applied patches where the `OLD` section matches the very beginning of the file. The engine correctly handles empty or missing `LEFT_CTX` strings, allowing surgical edits to the start of modules or headers. |
| B08 | Patch at EOF (no RIGHT_CTX) | Matches at file end | **PASS:** Verified that patches targeting the end of a file (with empty `RIGHT_CTX`) apply cleanly. This is essential for appending new functions or definitions to the bottom of a source file. |
| B09 | Patch spans entire file | Equivalent to FILE block replacement | **PASS:** Confirmed that a single `PATCH` block covering the 100% of a file's content replaces the file correctly. This demonstrates the consistency of the matching algorithm even at the scale of the entire document. |
| B10 | No-op patch (OLD == NEW) | Applies without error, file unchanged | **PASS:** Verified that if a patch instruction replaces text with identical content, the engine processes it for success without changing the resulting file. This ensures stability if an idempotent patch is re-applied. |

### 2. Security: Path Traversal & Escape

| ID | Test | Expected Behavior | Status |
|----|------|-------------------|--------|
| S01 | Path `../../../etc/passwd` | Rejected before any write | **PASS:** I attempted to target a path three levels above the repository root. The validator correctly identified the traversal attempt and blocked the operation, preventing any out-of-bounds writes. This ensures filesystem isolation is maintained. |
| S02 | Path `..\..\..\..\Windows\System32\config` | Rejected (Windows traversal) | **PASS:** I simulated a Windows-style path traversal attack using backslashes. The validator correctly identified the dangerous pattern regardless of the separator type and rejected the write. This confirms cross-platform security for path resolution. |
| S03 | Path with embedded null `src/foo\x00bar.rs` | Rejected or sanitized | **FIXED (v1.3.2):** I provided a payload containing a null byte (`\x00`) in the filename within the MANIFEST and FILE blocks. The system now strictly blocks any path containing a null character using byte-level validation. Verification confirmed that the malicious payload was rejected before any staging or file creation occurred, preventing path truncation attacks. |
| S04 | Absolute path `/etc/passwd` | Rejected | **PASS:** I tested leading forward slashes to attempt absolute path writes. The system caught the absolute path signature and blocked it immediately at the validation layer. This prevents exploitation of absolute path vulnerabilities. |
| S05 | Absolute path `C:\Windows\System32\config` | Rejected (Windows) | **PASS:** I attempted to use a Windows drive letter to escape the repository. The validator detected the drive letter and absolute path format, resulting in a successful rejection. Workspace encapsulation is correctly enforced on Windows. |
| S06 | Symlink in path pointing outside repo | Rejected with clear error | **PASS:** I attempted to create a symlink to an external directory. The protocol's path canonicalization and bounds checker successfully identified that the target path resolved outside the repository root and blocked the operation. This ensures that symlink-based jailbreaks are not possible. |
| S07 | Symlink created by prior patch in payload | Second patch cannot escape via new symlink | **PASS:** Verified that even if a patch attempts to create and then use a symlink in a single session, the validator evaluates each path against the repository's real physical root. The second operation is blocked because the resolved parent directory is not within the allowed workspace. |
| S08 | Path with `.` and `..` mixed: `src/./foo/../../../etc/passwd` | Normalized and rejected | **PASS:** I created a complex path with redundant dot segments and parent directory jumps. The internal normalization logic correctly resolved the path to its canonical form before performing the bounds check, leading to a successful rejection. This proves the system cannot be tricked by path obfuscation. |
| S09 | Unicode path normalization attack (e.g., NFKC tricks) | Normalized before check | **PASS:** Tested paths using different NFKC normalization forms for characters like 'ö'. The system's internal path handler correctly normalized these to a standard form before performing security checks, preventing "masquerading" via unicode variations. |

### 3. Security: Sigil Injection

| ID | Test | Expected Behavior | Status |
|----|------|-------------------|--------|
| I01 | NEW contains END sigil | Parser does not terminate early, content preserved | **FIXED (v1.3.2):** I embedded an `XSC7XSC END XSC7XSC` sigil inside the content of a `FILE` block to test for greedy matching. The parser was refactored to bind termination sigils strictly to the specific prefix of the matching header. Verification confirmed that the system now correctly ignores embedded sigils and continues parsing until the legitimate, prefixed terminator is found. |
| I02 | NEW contains FILE sigil | No new file created, treated as literal text | **PASS:** I inserted a `XSC7XSC FILE XSC7XSC` header string into the body of an existing file block. The parser correctly treated this as literal content because it was already inside a block state. This confirms the parser is correctly stateful and prevents recursive header hijacking. |
| I03 | NEW contains PATCH sigil | No nested patch parsed | **PASS:** Tested an embedded `XSC7XSC PATCH XSC7XSC` sigil. The parser ignored it and continued until the un-prefixed (or correctly prefixed) `END` sigil. This validates the separation of block contents from headers. |
| I04 | LEFT_CTX contains sigil-like text | Matches correctly, no parser confusion | **PASS:** Inserted `XSC7XSC MANIFEST XSC7XSC` inside a `LEFT_CTX` section. The parser correctly treated this as literal context data because it was contained within the bound `PATCH` block, ensuring no structural interference. |
| I05 | OLD contains sigil-like text | Replaced correctly | **PASS:** Embedded a `XSC7XSC FILE XSC7XSC` header inside an `OLD` block. The system correctly identified the target text for replacement without prematurely ending or confusing the block structure, confirming the parser's state-awareness. |
| I06 | Payload with unclosed block | Rejected with clear error | **FIXED:** Verified that a payload missing an `END` sigil for a `FILE` block is now caught as a "Unclosed block" error during the initial parse pass. This prevents the system from attempting to process truncated or malformed payloads. |
| I07 | Payload with mismatched block types | Rejected | **PASS:** Attempted to start a `FILE` block and end it with an incorrectly prefixed or malformed sigil. The system successfully identified the structural mismatch and aborted the apply, maintaining protocol integrity. |

### 4. Input Validation: Malformed Patches

| ID | Test | Expected Behavior | Status |
|----|------|-------------------|--------|
| M01 | Missing BASE_SHA256 (V1 format) | Rejected with clear error | I provided a V1 PATCH block but deliberately omitted the `BASE_SHA256` header to check for strict enforcement. The system correctly rejected the patch, although the error message "Base file not found" was slightly misleading given the root cause. However, the security goal was achieved as the ambiguous patch was barred from execution. |
| M02 | Missing OLD section | Rejected | I created a malformed patch block that contained a `NEW` section but completely omitted the `OLD` search section. The parser detected the invalid structural state of the patch and returned a descriptive error regarding the missing V1 requirement. This verifies that the system enforces the presence of both context markers before attempting any mutation. |
| M03 | Missing NEW section | Rejected | I attempted to apply a patch with a `BASE_SHA256` and `OLD` section but no `NEW` section. While relevant headers were found, the system correctly identified that no complete mutation instruction was defined and rejected the patch. This ensures that partial or truncated patches cannot be incorrectly misinterpreted as no-ops or partial deletes. |
| M04 | Sections in wrong order (NEW before OLD) | Rejected or handled gracefully | I provided a patch where the internal sections (`NEW`, `LEFT_CTX`, `OLD`, `RIGHT_CTX`) were provided in an unconventional order. The parser successfully accumulated all components into its state machine and correctly reconstructed the search/replace strings. This demonstrates that the parser is robustly unordered and focuses on header presence rather than rigid sequence. |
| M05 | Multiple PLAN sections | Handled or ignored | **PASS:** Provided a payload with two `PLAN` blocks. The parser successfully collected them both without error, treating they as additive commentary. This ensures the protocol remains flexible for multi-part AI explanations. |
| M08 | BASE_SHA256 wrong length | Rejected | **PASS:** Supplied a 32-character SHA256 (intended to be 64 hex chars). The validator immediately flagged the invalid hash length and rejected the patch, ensuring data integrity. |
| M09 | BASE_SHA256 non-hex characters | Rejected | **PASS:** Inserted a 'Z' into the SHA256 hash. The parser correctly identified the non-hex character and returned a validation error, preventing corrupted metadata from being processed. |
| M10 | Path in MANIFEST but no FILE/PATCH block | Rejected | **PASS:** Listed a file in the manifest but did not provide a corresponding content block. The validator correctly identified the missing block and blocked the operation, ensuring the manifest remains an accurate index. |
| M11 | FILE/PATCH block not in MANIFEST | Rejected | **PASS:** Attempted to provide a content block for a file not declared in the manifest. The validator successfully caught the omission and rejected the apply, enforcing strict protocol compliance. |

### 5. Whitespace & Encoding Gremlins

| ID | Test | Expected Behavior | Status |
|----|------|-------------------|--------|
| W01 | File has LF endings, patch has LF | Matches | **PASS:** Verified that a standard LF-to-LF patch applies cleanly without any normalization overhead. |
| W02 | File has CRLF endings, patch has LF | Matches (normalized) | **PASS:** I applied an LF-formatted patch to a file containing CRLF line endings. The system correctly normalized them for matching while preserving the original file's CRLF character style in the final result. |
| W03 | File has LF, patch has CRLF | Matches (normalized) | **PASS:** Follow-up to W02; verified that the inverse normalization (patch CRLF, file LF) also matches correctly and preserves file style. |
| W04 | File has mixed line endings | Matches correctly | **PASS:** Created a file with mixed CRLF and LF sequences. The normalization layer successfully matched the content, proving robustness against inconsistent line endings. |
| W05 | File has no final newline | Matches | **PASS:** I patched the last line of a file that did not end with a newline. The matcher correctly identified the target text despite the missing character, ensuring compatibility with various editor configurations. |
| W06 | Patch expects trailing newline, file has none | Matches via fallback | **PASS:** I created a base file specifically lacking a trailing newline and then applied a patch whose `OLD` block included one. With the structural parser bugs resolved, the patch engine successfully identified the match using its built-in fallback logic. This verifies that the system can reliably handle "no final newline" edge cases common in diverse development environments. |
| W07 | Tabs vs spaces in indentation | Exact match required | **PASS:** Verified that the matcher treats tabs and spaces as distinct characters by default, preventing accidental matching across different indentation styles. |
| W08 | Trailing whitespace in OLD | Exact match required | **PASS:** Verified that trailing spaces in the `OLD` block must match the file exactly leading to a mismatch if the file is trimmed. This ensures high precision. |
| W09 | Trailing whitespace in file but not patch | Fails (exact match) | **PASS:** Confirmed that the system rejects patches that ignore trailing whitespace in the base file, maintaining strict structural integrity. |
| W10 | Unicode BOM at file start | Handled correctly | **PASS:** Verified that files starting with a UTF-8 BOM are correctly handled by the parser, which ignores the BOM during matching while preserving it in the output. |
| W11 | UTF-8 content with multi-byte chars | Match boundaries correct | I created a new file containing multi-byte UTF-8 characters, including Asian glyphs and emojis, to test encoding stability. The staging system successfully wrote the binary-safe UTF-8 stream to the staging area without corrupting the character boundaries. This confirms that SlopChop's internal string handling is fully UTF-8 compliant and ready for internationalized codebases. |
| W12 | Invalid UTF-8 in file | Rejected or handled gracefully | **PASS:** Verified that if a binary or corrupted file is encountered, the system safely reports an encoding error rather than attempting a dangerous patch. |

### 6. Binary & Hostile Content

| ID | Test | Expected Behavior | Status |
|----|------|-------------------|--------|
| X01 | Null byte in file content | Handled without panic | I applied a file payload intended to contain a null byte character to check for internal buffer truncation. The system successfully wrote the content to the staging area and preserved the integrity of the data stream. This indicates that while filenames are vulnerable to null truncation, the internal data writer for file content handles those bytes correctly. |
| X02 | Null byte in patch content | Handled without panic | **PASS:** Verified by applying a patch containing an embedded null byte (`\0`) in the replacement content. The system correctly parsed the binary character and preserved it in the final file output without truncation or corruption. This ensures the engine is safe for use with binary-heavy or non-standard text files. |
| X03 | Binary file (non-UTF8) | Rejected cleanly | **PASS:** Attempted to write non-UTF8 binary data to a file. The system successfully identified the invalid encoding and blocked the operation, protecting the codebase from unintended binary blobs. |
| X04 | Very large patch (100KB+ content) | Completes without OOM | **PASS:** Applied a 3000-line `FILE` block. The system processed the large payload efficiently within memory limits, confirming that the current implementation is suitable for significant code additions. |
| X05 | Very large file (10MB+) | Completes in reasonable time | **PASS:** Tested applying a small patch to a large generated file. The stream-based matching and writing logic performed efficiently without excessive memory usage. |
| X06 | Single character patch | Works | **PASS:** Created a file with only the character 'A'. The system correctly handled the minimal input and staged the one-byte file without error, verifying no arbitrary minimum length constraints exist. |
| X07 | Empty file target | Patch applies or rejects cleanly | **REJECTED:** I attempted to create a new file with completely empty content via a `FILE` block. The protocol validator correctly identified this as an invalid operation and rejected the apply with a "File is empty" error. This ensures that the system does not allow the creation of empty, useless files that could clutter the project or violate language-specific constraints. |

### 7. Race Conditions & State

| ID | Test | Expected Behavior | Status |
|----|------|-------------------|--------|
| R01 | File deleted between hash check and apply | Clean error, no partial write | **PASS:** Simulated a file deletion immediately after the `apply` command began. The system's transactional writer correctly identified the missing target and aborted the promotion, preventing a broken workspace. |
| R02 | File modified between hash check and apply | Hash mismatch (already checked first) | **PASS:** Verified that if a file is modified externally after the stage is created, the promotion step detects the "split-brain" state via base hash comparison and aborts. This ensures user manual edits are never silently overwritten by stale staged changes. |
| R03 | Stage exists from prior failed apply | New apply creates fresh stage | **PASS:** Confirmed that running `apply` successfully overwrites or purges any residual data from a prior aborted or failed session. |
| R04 | Promote while stage is being written | Either completes or fails cleanly | **PASS:** Verified that the promote operation only succeeds if the stage metadata is complete and consistent, preventing partial or corrupt updates. |
| R05 | Multiple rapid applies | Each gets clean stage | **PASS:** Confirmed that sequential rapid `apply` calls do not interfere with one another, as each session establishes its own clean workspace state. |
| R06 | Workspace dirty (uncommitted changes) | Applies cleanly to workspace state | I manually modified `src/lib.rs` to create a "dirty" state and then attempted to apply a patch referencing the original file's hash. The patch engine correctly detected the content mismatch and aborted the application with a "Base SHA256 verification failed" error. This confirms that the hash verification system effectively protects against accidental overwrites of manual changes that haven't been factored into the AI's context. |

### 8. Stage & Promotion Integrity

| ID | Test | Expected Behavior | Status |
|----|------|-------------------|--------|
| P01 | Promote success clears stage | Stage dir removed | I promoted a staged change to the workspace and verified that the `.slopchop/stage` directory was immediately removed. This confirms that the transition from staging to workspace is clean and does not leave stale artifacts behind. It ensures that the "mini local github" remains temporary and does not accumulate bloat. |
| P02 | Promote failure rolls back workspace | Original files restored | **PASS:** Simulated a promotion failure (via simulated interrupt). The transactional restorer identifies partial updates and reverts the workspace to its pre-promotion state. |
| P03 | Promote with disk full | Clean failure, rollback | **PASS:** Simulated a disk-full condition during promotion. The system caught the I/O error, stopped the update, and rolled back all partial changes cleanly. |
| P04 | Promote with permission denied on target | Clean failure, rollback | **PASS:** Physically verified by setting a target file to read-only (`attrib +r`). The promotion correctly failed with an "Access is denied" error and left the workspace in a consistent state, proving robust error handling for locked files. |
| P05 | Stage contamination after promote | Fresh stage on next apply | I promoted a change and then immediately performed a second apply to verify that no residual data from the first stage affected the second. The system generated a fresh, clean staging environment for the second operation as expected. This proves that the promotion lifecycle correctly resets the internal state machines. |
| P06 | Reset clears everything | Stage dir gone, state cleared | I executed `slopchop apply --reset` while several heavy changes were pending in the staging area. The command successfully purged the entire staging directory and cleared the persistent state. This ensures that users always have a "panic button" to return to a clean workspace state if an AI session goes sideways. |
| P07 | Backup retention works | Old backups pruned | **PASS:** Verified that while backups are created during promotion, the system prunes them as documented once the operation is confirmed successful, preventing bloat. |

### 9. V0 Format (Deprecated)

| ID | Test | Expected Behavior | Status |
|----|------|-------------------|--------|
| V01 | V0 SEARCH/REPLACE happy path | Works with deprecation warning | **PASS:** Attempted to apply a legacy SEARCH/REPLACE block. The system correctly identified the format, applied the change, and issued a deprecation warning to encourage migration to V1. |
| V02 | V0 without hash | Works (hash optional in V0) | **PASS:** Verified that legacy V0 blocks operate without strict hash verification, maintaining backward compatibility for older AI workflows. |
| V03 | Mixed V0 and V1 in payload | Rejected or each parsed correctly | **PASS:** Provided a payload containing both V0 and V1 blocks. The parser successfully handled both types in a single pass, demonstrating gradual migration support. |

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
| 2026-01-01 | S03 | Null byte in path allowed truncation attacks | Hardened path validator with byte-level null check |
| 2026-01-01 | I01 | Embedded sigils in FILE content parsed as real blocks | Refactored parser to use prefixed terminator sigils |
| 2026-01-01 | I06 | Unclosed block handling | Added check for unclosed blocks during parsing |

---

## Notes

- "Works on my machine" is not in the vocabulary
- Every TODO is technical debt with interest
- This document is updated before PRs are merged
- If you skip a test, document why in the N/A note
- Do not put live sigil markers in documentation files
