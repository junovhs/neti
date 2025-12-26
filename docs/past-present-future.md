# Past / Present / Future
**Status:** Canonical (living snapshot)  
**Last updated:** 2025-12-25  
**Canonical policy:** This document states the current operational reality and the single next action.  
**For full v1.0 priorities / blockers / DoD:** see `/docs/v1-brief.md`.

---

## 1) Past (What changed recently)

**v0.9.0 shipped.** The architectural pivot is complete and validated:
- **Staged workspace:** `slopchop apply` writes to `.slopchop/stage/worktree` (not the real repo).
- **Transactional promote:** `slopchop apply --promote` promotes verified changes to the real workspace (scoped to touched paths, with backup/rollback).
- **Parser hardening:** strict block validation + reserved-name protection.
- **Surgical patching (v0.9):** strict PATCH application with base hash enforcement and ambiguity rejection (current shipped format is strict SEARCH/REPLACE).

---

## 2) Present (Where we are right now)

**Status:** OPERATIONAL / HARDENING (v0.9.0 released)

### Operator-visible contract (current behavior)
- `slopchop apply` writes into stage; supports `FILE` and `PATCH`.
- `slopchop check` runs against stage if present; otherwise workspace.
- `slopchop apply --promote` promotes touched paths from stage → workspace transactionally.

### Trust boundary posture (current)
- Conservative failure: rejects ambiguity/staleness instead of guessing.
- Default writes are confined to the stage.
- Promotion is bounded and reversible on failure.

---

## 3) Future (What we do next)

We are in **Phase 3 (Polish) → v1.0.0**.

### Phase 3A: Patch UX & Diagnostics (NEXT OBJECTIVE)
Patch failures are safe but blunt. v1.0 requires deterministic, bounded, actionable diagnostics:
- “Did you mean?” suggestions (diagnostic locate only; never fuzzy-apply).
- Bounded visual diff summaries for match failures.
- Clear “NEXT:” instructions (repack/regenerate patch vs send full FILE).

### v1.0 Protocol Upgrade (PATCH format)
- Canonicalize PATCH to **context-anchored** form (LEFT_CTX / OLD / RIGHT_CTX / NEW).
- Retain v0.9 SEARCH/REPLACE PATCH only as deprecated compatibility (optional), without expanding the command surface.

(Full spec and priorities live in `/docs/v1-brief.md`.)

---

## Immediate Next Action
Begin **Phase 3A (Patch UX & Diagnostics)**.
