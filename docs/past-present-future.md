# Past / Present / Future

**Status:** Canonical (living snapshot)
**Last updated:** 2025-12-26
**Canonical policy:** This document states the current operational reality and the single next action.

---

## 1) Past (What changed recently)

**v1.1.0 Released (Post-Release Polish).**

* **Transport Hardening:** `apply` now handles indented/quoted sigil lines and strips UI-injected markdown fences (`--sanitize` is default for clipboard).
* **Enhanced Diagnostics:** `PATCH` failures show bounded visual diffs and "Did you mean?" suggestions.
* **Resilience:** Parser and Patch engine are robust against common chat UI artifacts.

**v1.0.0 Released.**

* **Trust Boundary:** Complete staging, verification, and promotion architecture.
* **Protocol:** Canonical `XSC7XSC` with context-anchored patching and SHA256 locking.
* **Observability:** Structured event logging and standardized exit codes.
* **Hygiene:** Zero Git dependency; clean 3 Laws compliance.

---

## 2) Present (Where we are right now)

**Status:** STABLE / PRODUCTION

### Operator-visible contract

* `slopchop apply` (Stage + Sanitize by default)
* `slopchop check` (Verify)
* `slopchop apply --promote` (Commit)
* `slopchop pack` (Context)

### Trust boundary posture

* System is a self-contained, high-integrity gatekeeper.
* Input layer is now hardened against hostile transport (markdown/chat UIs).

---

## 3) Future (What we do next)

We are in **Post-v1.0 Era**.

### Objectives

* Distribution (Homebrew / Scoop / Winget).
* Build useful things *with* SlopChop.

### Immediate Next Action

**Distribute.**