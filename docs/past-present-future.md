# Past / Present / Future

**Status:** Canonical (living snapshot)
**Last updated:** 2025-12-27
**Canonical policy:** This document states the current operational reality and the single next action.

---

## 1) Past (What changed recently)

**v1.1.x: The Trust Boundary is Sealed.**

*   **Transactional Staging:** `apply` creates a shadow sandbox; `promote` moves changes to the workspace with automatic rollback.
*   **Protocol Hardening:** The `XSC7XSC` sigil parser now strips transport noise (indentation, blockquotes) and sanitizes UI-injected markdown fences.
*   **Surgical Precision:** `PATCH` blocks are cryptographically locked via `BASE_SHA256` (now provided by `pack`) and offer visual diff diagnostics on mismatch.
*   **Zero-Dependency Hygiene:** Removed all Git coupling; SlopChop manages its own integrity state.

---

## 2) Present (Where we are right now)

**Status:** TRANSITIONING (Enforcement to Intelligence)

### Operator-visible contract

*   `slopchop apply` (Stage + Decontaminate)
*   `slopchop check` (Verify stage or workspace)
*   `slopchop apply --promote` (Atomic commit)
*   `slopchop pack --focus <file>` (High-density context with SHA snapshots)

### System Posture

The system is a proven gatekeeper. We are now moving from **Safety** (preventing breakage) to **Topology** (preventing complexity sprawl). We are implementing the **Law of Locality** to optimize the codebase for both human cognitive load and AI attention.

---

## 3) Future (What we do next)

We are entering the **Architectural Integrity Era (v1.2.x)**.

### Objectives

*   **The Law of Locality:** Introduce a metric-backed scan that penalizes "Sideways Dependencies" (spaghetti cables in the server rack).
*   **Stability Metrics:** Implement Martin's Instability ($I$) and Skew ($K$) signals to distinguish between stable hubs and volatile leaf features.
*   **Cognitive Load Optimization:** Limit "Dependency Distance" to ensure related logic stays in physical proximity.
*   **Distribution:** Prepare Homebrew/Scoop/Winget taps for the stable core.

### Immediate Next Action

**Implement the Stability Classifier.**
Compute fan-in ($C_a$) and fan-out ($C_e$) for all files to identify "Stable Hubs" that are safe for long-range dependencies vs "Leaf Features" that must remain local.