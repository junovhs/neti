# Stage → Branch Migration Checklist (some of this may be done but definitely not completely so just go through the steps)

## Phase 0: Hotfix UTF-8 Panic (Do Immediately)

In `src/apply/verification.rs`, fix the string truncation:

**Find:**
```rust
let truncated = if combined.len() > 1000 {
    format!("{}...\n[truncated]", &combined[..1000])
```

**Replace with:**
```rust
let truncated = if combined.len() > 1000 {
    let safe_end = floor_char_boundary(&combined, 1000);
    format!("{}...\n[truncated]", &combined[..safe_end])
```

**Add helper (or import from diagnostics.rs):**
```rust
fn floor_char_boundary(s: &str, mut idx: usize) -> usize {
    if idx >= s.len() { return s.len(); }
    while !s.is_char_boundary(idx) {
        idx = idx.saturating_sub(1);
    }
    idx
}
```

Commit as v1.4.1 hotfix.

## Phase 1: Add branch.rs

1. Copy `branch.rs` to `src/branch.rs`
2. Add to `src/lib.rs`:
   ```rust
   pub mod branch;
   ```
3. Run `slopchop check` — should still pass

## Phase 2: Add new CLI commands

In `src/cli/args.rs`, add to Commands enum:
```rust
/// Create or reset the work branch for AI editing
Branch {
    #[arg(long, short)]
    force: bool,
},

/// Merge work branch to main
Promote {
    #[arg(long)]
    dry_run: bool,
},

/// Abort work branch and return to main
Abort,
```

In `src/cli/handlers.rs`, add handlers:
```rust
pub fn handle_branch(force: bool) -> Result<SlopChopExit> {
    match crate::branch::init_branch(force)? {
        crate::branch::BranchResult::Created => {
            println!("Created branch 'slopchop-work'. You're now on the work branch.");
        }
        crate::branch::BranchResult::Reset => {
            println!("Reset branch 'slopchop-work'. Fresh start.");
        }
        crate::branch::BranchResult::AlreadyOnBranch => {
            println!("Already on 'slopchop-work'.");
        }
    }
    Ok(SlopChopExit::Success)
}

pub fn handle_promote(dry_run: bool) -> Result<SlopChopExit> {
    match crate::branch::promote(dry_run)? {
        crate::branch::PromoteResult::DryRun => {
            println!("[DRY RUN] Would merge 'slopchop-work' into main.");
        }
        crate::branch::PromoteResult::Merged => {
            println!("Merged 'slopchop-work' into main. Work branch deleted.");
        }
    }
    Ok(SlopChopExit::Success)
}

pub fn handle_abort() -> Result<SlopChopExit> {
    crate::branch::abort()?;
    println!("Aborted. Work branch deleted, back on main.");
    Ok(SlopChopExit::Success)
}
```

## Phase 3: Update advisory nag

In `src/apply/verification.rs` or wherever the nag lives, change:
```rust
// Old: reads from stage state.json
// New: reads from git status
let modified_count = crate::branch::count_modified_files();
if modified_count > 3 {
    // Show advisory
}
```

## Phase 4: Delete stage AND sabotage

Remove these files:
```
src/stage/copy.rs
src/stage/manager.rs
src/stage/mod.rs
src/stage/promote.rs
src/stage/state.rs
src/stage/sync.rs
src/analysis/sabotage.rs
```

Remove from `src/lib.rs`:
```rust
pub mod stage;  // DELETE THIS LINE
```

Remove from `src/analysis/mod.rs`:
```rust
pub mod sabotage;  // DELETE THIS LINE
```

Remove any imports of `crate::stage::*` and `crate::analysis::sabotage` throughout codebase.

Note: `sabotage` was the manual proof-of-concept for mutation testing. Now that `src/mutate/` exists, it's redundant.

## Phase 5: Update CLI dispatch

Remove old commands:
- `Stage { force }` 
- `Apply { sync, reset, ... }`

Update help text and README.

## Phase 6: Deprecate old workflow

Update `.agent/rules/slopchop.md` and `.agent/workflows/dev.md`:
```
Old: slopchop stage --force && cd .slopchop/stage/worktree
New: slopchop branch
```

## Phase 7: Clean up

- Delete `.slopchop/stage/` directory
- Update `.gitignore` (remove `.slopchop/` line if not needed)
- Run `slopchop check`
- Commit as v1.5.0

## Line Count

| Action | Lines |
|--------|-------|
| Delete stage/* | -1700 |
| Delete sabotage.rs | -120 |
| Add branch.rs | +150 |
| Add handlers | +50 |
| **Net** | **-1620** |
