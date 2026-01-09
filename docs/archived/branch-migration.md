# Stage → Branch Migration Checklist (COMPLETED)

**Status:** ARCHIVED
**Date:** 2026-01-08

## Phase 0: Hotfix UTF-8 Panic (Done)
- Fixed `generate_ai_feedback` panic with `floor_char_boundary`.

## Phase 1: Add branch.rs (Done)
- Added `src/branch.rs` for git operations.

## Phase 2: Add new CLI commands (Done)
- Added `Branch`, `Promote`, `Abort` to `src/cli/args.rs`.
- Implemented handlers in `src/cli/handlers.rs`.
- Refactored dispatch in `src/cli/dispatch.rs`.

## Phase 3: Update advisory nag (Done)
- `src/apply/advisory.rs` now uses `branch::count_modified_files()`.

## Phase 4: Delete stage AND sabotage (Done)
- Deleted `src/stage/`.
- Deleted `src/analysis/sabotage.rs`.

## Phase 5: Update CLI dispatch (Done)
- Wired up new commands in `src/bin/slopchop.rs`.

## Phase 6: Deprecate old workflow (Done)
- Updated `README.md` and `slopchop.toml`.

## Phase 7: Clean up (Done)
- Manual cleanup of `.slopchop/stage` performed by user/system.