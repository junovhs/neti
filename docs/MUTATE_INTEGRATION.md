# Mutation Testing Integration Plan

**Status**: NOT INTEGRATED  
**Last Updated**: 2026-01-07  
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

The `src/mutate/` module EXISTS but is NOT wired to CLI.

Files present:
- `src/mutate/mod.rs` — Orchestration
- `src/mutate/mutations.rs` — Mutation types
- `src/mutate/discovery.rs` — AST scanning
- `src/mutate/runner.rs` — Test execution
- `src/mutate/report.rs` — Output formatting

---

## Integration Steps

### Step 1: Add to lib.rs

In `src/lib.rs`, add:
```rust
pub mod mutate;
```

### Step 2: Add CLI command

In `src/cli/args.rs`, add to `Commands` enum:
```rust
/// Run mutation testing to find test gaps [EXPERIMENTAL]
Mutate {
    /// Test timeout in seconds
    #[arg(long, default_value = "30")]
    timeout: u64,
    /// Output results as JSON
    #[arg(long)]
    json: bool,
    /// Filter files by path pattern
    #[arg(long, short)]
    filter: Option<String>,
    /// Stop on first surviving mutant
    #[arg(long)]
    fail_fast: bool,
},
```

### Step 3: Add handler

In `src/cli/handlers.rs`, add import:
```rust
use crate::mutate::{self, MutateOptions};
```

Add handler function:
```rust
/// Handles the mutate command.
///
/// # Errors
/// Returns error if mutation testing fails.
pub fn handle_mutate(
    timeout: u64,
    json: bool,
    filter: Option<String>,
    fail_fast: bool,
) -> Result<SlopChopExit> {
    let opts = MutateOptions {
        timeout_secs: timeout,
        json,
        filter,
        fail_fast,
    };
    
    let repo_root = get_repo_root();
    let report = mutate::run(&repo_root, &opts)?;
    
    if report.summary.survived > 0 {
        Ok(SlopChopExit::CheckFailed)
    } else {
        Ok(SlopChopExit::Success)
    }
}
```

### Step 4: Add dispatch

In `src/cli/handlers.rs` main dispatch, add:
```rust
Some(Commands::Mutate { timeout, json, filter, fail_fast }) => {
    handle_mutate(timeout, json, filter, fail_fast)
}
```

---

## Safety Requirements Before Running

### Pre-flight checklist
```bash
# 1. Create a branch
git checkout -b mutation-test-run

# 2. Verify clean state
git status  # Should be clean
slopchop check  # Should pass

# 3. Run mutation testing
slopchop mutate --filter src/tokens.rs --timeout 30

# 4. Verify files restored
git diff  # Should be empty

# 5. If clean, return to main
git checkout main
git branch -d mutation-test-run
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

## Future Improvements

| Feature | Priority | Difficulty |
|---------|----------|------------|
| Timeout per mutation | HIGH | Easy |
| `--fail-fast` flag | HIGH | Easy |
| Return value mutations | MEDIUM | Medium |
| Automatic git stash/restore | HIGH | Easy |
| Parallel execution | LOW | Hard (needs workspace copies) |

---

## Usage (Once Integrated)

```bash
# Quick test on single file
slopchop mutate --filter src/tokens.rs

# Full codebase (takes a while)
slopchop mutate

# Stop on first survivor
slopchop mutate --fail-fast

# JSON output for CI
slopchop mutate --json
```
