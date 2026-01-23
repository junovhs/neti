# MUTATION-DAILY: 2026-01-23

**Context:** SlopChop is in a MUTATION-FREEZE. All feature work paused until 80% kill rate.

**Current State:**
- Kill Rate: **20.6%** (385 caught / 1,870 testable)
- Target: **80%** (1,496 caught)
- Gap: **1,111 mutants to kill**

**Reference Docs:**
- `MUTATION-FREEZE.md` — Full plan with all test cases
- `.cargo/mutants.toml` — Exclusions config

---

## Today's Focus: Complete Phase 1 Gaps + Start Phase 2

### Priority 1: Phase 1 Gaps (36 remaining test cases)

These modules have tests but incomplete coverage. Fill the gaps:

**pack/focus.rs** (5 gaps)
```
TC-FOCUS-06: Depth=2 includes transitive dependencies
TC-FOCUS-07: Circular dependencies don't infinite loop
TC-FOCUS-08: Missing files handled gracefully
TC-FOCUS-09: Budget overflow truncates peripheral, not focal
TC-FOCUS-10: Empty graph returns just focal file
```

**graph/rank/pagerank.rs** (4 gaps)
```
TC-PR-07: Disconnected components handled
TC-PR-08: Personalization boosts anchor's neighbors
TC-PR-09: Empty graph returns empty map
TC-PR-10: Self-loops handled correctly
```

**graph/locality/distance.rs** (4 gaps)
```
TC-DIST-02: Sibling files = distance 2
TC-DIST-04: Cousin files (shared grandparent) = distance 4
TC-DIST-05: Deeply nested vs root = correct count
TC-DIST-08: Non-existent paths don't panic
```

**graph/locality/cycles.rs** (6 gaps)
```
TC-CYC-05: Three-node cycle detected
TC-CYC-06: Multiple disjoint cycles all found
TC-CYC-07: Figure-8 (two cycles sharing node) handled
TC-CYC-08: Large cycle (10+ nodes) detected
TC-CYC-09: DAG with convergence (not a cycle) passes
TC-CYC-10: Cycle membership is complete (all nodes listed)
```

**apply/validator.rs** (6 gaps)
```
TC-VAL-10: DELETE operation on missing file = warning
TC-VAL-11: UPDATE operation validates original exists
TC-VAL-12: Whitespace-only paths rejected
TC-VAL-13: Valid special characters in paths allowed
TC-VAL-14: Symlink paths handled safely
TC-VAL-15: Case sensitivity respected (or not, per OS)
```

**apply/parser.rs** (8 gaps)
```
TC-PARSE-13: Path extraction from FILE header
TC-PARSE-14: Whitespace handling in content
TC-PARSE-15: Very large block (100KB) handled
TC-PARSE-16: Binary-looking content preserved
TC-PARSE-17: Unicode in path and content
TC-PARSE-18: Windows line endings (CRLF)
TC-PARSE-19: Mixed line endings
TC-PARSE-20: Interleaved PLAN and FILE blocks
```

**skeleton.rs** (2 gaps)
```
TC-SKEL-07: Comments and attributes preserved
TC-SKEL-09: Multiline signatures preserved
```

**tokens.rs** (1 gap - optional)
```
TC-TOK-08: Large text performance (sanity check, not assertion)
```

---

### Priority 2: Start Phase 2 — Pattern Detectors

If Phase 1 gaps are done, begin testing V2 pattern detectors. Start with:

**patterns/state.rs** (S01, S02, S03) — ~20 mutants
- Test `static mut` detection
- Test exported mutable detection  
- Test suspicious global container detection

**patterns/security.rs** (X01-X07) — ~25 mutants
- SQL injection detection
- Command injection detection
- Hardcoded secrets detection
- Dangerous TLS config detection

See `MUTATION-FREEZE.md` Phase 2 for full test case specifications.

---

## Verification Commands

```bash
# Run tests
cargo test

# Check SlopChop self-scan
slopchop check

# Quick mutation check on specific file
cargo mutants -f src/tokens.rs -j 4

# Full mutation run (after completing work)
cargo mutants --package slopchop -j 4
```

---

## Success Criteria for Today

- [ ] All 36 Phase 1 gap tests written and passing
- [ ] At least `patterns/state.rs` tests complete
- [ ] `cargo test` passes
- [ ] `slopchop check` green
- [ ] Run `cargo mutants` at end of session, record new kill rate

**Expected kill rate after today: ~30-35%**
