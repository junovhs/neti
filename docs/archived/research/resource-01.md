# SOTA Report: Event and Subscription Leaks (Resource-01)

**Date**: January 2026  
**Subject**: Static and dynamic detection of memory leaks caused by improper lifecycle management of event listeners and observable subscriptions.

## 1. Executive Summary
The "Lapsed Listener Problem" remains a primary driver of memory exhaustion in garbage-collected environments. While dynamic analysis (e.g., LeapCanary) is common for identifying leaks after they occur, State-of-the-Art (SOTA) static analysis focuses on **Callback-Aware Control Flow Graphs** and **Lifecycle-Method Anchoring** to prevent leaks before they reach production.

---

## 2. The Lapsed Listener Problem
A memory leak occurs when a "Subject" (long-lived) maintains a strong reference to an "Observer" (short-lived) via an event registration, preventing the Observer from being collected.

*   **SOTA Detection Strategy**: **Registration-Symmetry Check**.
    *   Analyzers identify "Register" operations (e.g., `addEventListener`, `+= operator`) in component mount/initialization methods.
    *   They perform **Interprocedural Data Flow Analysis** to find a corresponding "Unregister" operation in the teardown/disposal methods.
    *   **Leading Indicator**: Any registration that lacks a symmetric unregistration on the "Dominator Path" to object destruction is flagged.

---

## 3. Tooling SOTA: JSWhiz and Relda2

### 3.1 JSWhiz (Google Closure Meta-Analysis)
Developed and used at Google for massive apps like Gmail.

*   **Mechanism**: Static analysis within the Closure compiler. It enforces a strict "Unlisten" protocol. If an event is registered, the compiler verifies that the listener's handle is stored and subsequently used in an `unlisten` call.
*   **Success**: It reduced the memory footprint of major web applications by up to 40% in some subsystems by catching "Forgotten Listeners."
*   **Citation**: (2014). *JSWhiz: Static Analysis for Memory Leak Detection*.

### 3.2 Relda2 (Android-Specific SOTA)
Targets the "Callback-Heavy" nature of Android development.

*   **Mechanism**: **Callback Graphs**.
    1.  Relda2 constructs a Function Call Graph augmented with Android lifecycle callbacks (`onCreate`, `onDestroy`).
    2.  It traces resource handles (including event handlers) through the lifecycle.
    3.  It identifies "Orphaned Callbacks" that remain registered after the Activity/Fragment context is destroyed.
*   **Citation**: (2018). *Light-Weight, Inter-Procedural and Callback-Aware Resource Leak Detection*.

---

## 4. ReactiveX (RxJS / Rx.NET) SOTA
In the Observable paradigm, leaks are caused by "Floating Subscriptions."

*   **SOTA Pattern**: **`takeUntil` Lifecycle Anchoring**.
    *   Instead of manual unsubscription, SOTA lint rules (e.g., for Angular/RxJS) enforce the use of `takeUntil(this.destroy$)`.
*   **Detection**: Static analysis flags any `.subscribe()` call that is not:
    1.  Returned and stored in a `Subscription` object.
    2.  Piped through a `takeUntil` or similar completion operator.
    3.  Awaited/converted via the `async` pipe.

---

## 5. Summary Of Analysis Metrics

| Metric | Grain | Purpose |
| :--- | :--- | :--- |
| **Registration Symmetry** | AST-Based | Basic "Forgotten Unsubscribe" Check |
| **Callback Graph Coverage** | Graph-Based | Deep Lifecycle Interaction Check |
| **Region Ownership** | Ownership-Based | Proving "Isolated Cleanup" |
| **Subscription Entropy** | Statistical | Identifying "Hot" Event Sources |

---

## 6. References
1.  **JSWhiz**: [Google Research: Static Analysis for JS Leaks](https://research.google/pubs/archive/42823/)
2.  **Relda2**: [ResearchGate: Resource Leak Detection for Android](https://www.researchgate.net/publication/224214227)
3.  **Lapsed Listener**: [Wikipedia: The Lapsed Listener Problem](https://en.wikipedia.org/wiki/Lapsed_listener_problem)
4.  **RxJS Linting**: [Angular Extensions for SOTA Subscription Management](https://github.com/angular-extensions/lint-rules)
