# Mutation Testing Integration Plan

**Status**: INTEGRATED (Alpha)
**Last Updated**: 2026-01-08
**Version**: v1.5.0
**Baseline Commit**: bf46c8e

---

## Lessons Learned (2026-01-07 Incident)

An unsupervised overnight mutation testing run corrupted the codebase:

| File | Mutation | Effect |
|------|----------|--------|
| `src/constants.rs` | `\|\|` → `&&` | Discovery returned 0 files |
| `src/apply/patch/diagnostics.rs` | `>=` → `<` | Infinite loop in tests |
| `src/lang.rs` | `==` → `!=` | Logic inversion |
| `src/types.rs` | `==` → `<=` | Clippy error |

**Root Cause**: `cargo-mutants` did not properly restore files after mutations. The corrupted state was committed without verification.

**Prevention Rules**:
1. NEVER run mutation testing unsupervised
2. ALWAYS run on a branch, not main
3. ALWAYS `git diff` before committing after mutation runs
4. ALWAYS run `slopchop check` to verify integrity

---

## Current State

The `src/mutate/` module is wired to the CLI as of v1.5.0.

Files present:
- `src/mutate/mod.rs` — Orchestration
- `src/mutate/mutations.rs` — Mutation types
- `src/mutate/discovery.rs` — AST scanning
- `src/mutate/runner.rs` — Test execution
- `src/mutate/report.rs` — Output formatting

---

## Integration Steps (Completed)

1. Added `pub mod mutate;` to `src/lib.rs`.
2. Added `Mutate` command to `src/cli/args.rs`.
3. Added handler to `src/cli/handlers.rs`.
4. Wired dispatch in `src/bin/slopchop.rs`.

---

## Safety Requirements Before Running

**Warning:** The mutation runner modifies files in place. It relies on internal restoration logic which has failed in the past.
**Requirement:** Only run `slopchop mutate` on a clean `slopchop-work` branch.

### Pre-flight checklist
```bash
# 1. Create a branch
slopchop branch

# 2. Verify clean state
git status  # Should be clean
slopchop check  # Should pass

# 3. Run mutation testing
slopchop mutate --filter src/tokens.rs --timeout 30

# 4. Verify files restored
git diff  # Should be empty

# 5. If clean, return to main
slopchop abort
```

### If mutations are left behind
```bash
git checkout -- .
```

---

## Comparison: slopchop mutate vs cargo-mutants

| Aspect | slopchop mutate | cargo-mutants |
|--------|-----------------|---------------|
| Speed | ~11s | ~66s |
| Cross-language | Rust, TS, Python | Rust only |
| Mutation types | Operators, booleans | Operators, booleans, return values |
| Integration | Native | External |
| File restoration | Manual (needs fix) | Unreliable |

---

## Known Limitations (v1)

1. **Serial execution** — No parallelism yet
2. **No return value mutations** — Only operators and booleans
3. **No automatic restoration** — Must verify with `git diff`
4. **No timeout enforcement** — Can hang on infinite loops

---

## Future Work

1. **Branch Safety**: Modify `src/mutate/mod.rs` to assert that it is running on a disposable branch (using `src/branch.rs`).
2. **Parallelism**: Implement workspace copying for parallel execution.

---

## Usage

```bash
# Quick test on single file
slopchop mutate --filter src/tokens.rs

# Full codebase (takes a while)
slopchop mutate

# JSON output for CI
slopchop mutate --json
```