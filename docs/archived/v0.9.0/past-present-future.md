# Past / Present / Future (Archived Snapshot)
**Status:** Archived (immutable)  
**Snapshot date:** 2025-12-25  
**Applies to:** v0.9.0 release state

---

## 1) Past (What changed recently)

**v0.9.0 shipped.** The architectural pivot is complete and validated:
- **Staged workspace:** `slopchop apply` writes to `.slopchop/stage/worktree` (not the real repo).
- **Transactional promote:** `slopchop apply --promote` promotes verified changes to the real workspace (scoped to touched paths, with backup/rollback).
- **Parser hardening:** strict block validation + reserved-name protection.
- **Surgical patching:** strict search/replace; rejects ambiguity and base/hash mismatches.
- **Green build:** tests passing, including patch precision/safety coverage.

---

## 2) Present (Where we are right now)

**Status:** OPERATIONAL / HARDENING (v0.9.0 released)

### Operator-visible contract (current behavior)
- `slopchop apply`  
  Writes into stage. Supports `FILE` and `PATCH`.
- `slopchop check`  
  Runs against stage if present; otherwise the real workspace.
- `slopchop apply --promote`  
  Promotes touched paths from stage → workspace transactionally.

### Trust boundary posture (current)
- Failures are conservative (rejects instead of guessing).
- No accidental writes outside stage by default.
- Promotion is bounded and reversible on failure.

---

## 3) Future (What we do next)

We are in **Phase 3 (Polish) → v1.0.0**.

### Phase 3A: Patch UX & Diagnostics (NEXT OBJECTIVE)
Patch failures are safe but blunt. We need deterministic, bounded, actionable diagnostics:
- “Did you mean?” suggestions (diagnostic locate only; never fuzzy-apply).
- Bounded visual diff summaries for match failures.
- Clear “NEXT:” instructions (repack/regenerate patch vs send full FILE).

---

## Immediate Next Action
Begin **Phase 3A (Patch UX & Diagnostics)**.
