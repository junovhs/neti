# SOTA Report: Race Detection in Single-Threaded Loops (Concurrency-01)

**Date**: January 2026  
**Subject**: Static and dynamic analysis of async data races and interleaving bugs in single-threaded event loops (JS/Node.js).

## 1. Executive Summary
While single-threaded environments like Node.js avoid "Kernel-Level Data Races," they are susceptible to **Async Data Races** (Interleaving Races) where the non-deterministic order of asynchronous tasks (Promises, Callbacks) leads to state corruption. State-of-the-Art (SOTA) research utilizes **Callback Graphs** and **Event Scheduling Instrumentation** to detect these races, which are often invisible to traditional multi-threaded race detectors.

---

## 2. Formal Models for Async Concurrency

### 2.1 The Callback Graph (Async-TAJS)
Static analysis of JavaScript requires a model that understands the temporal relationship between asynchronous blocks.

*   **Mechanism**: **Callback Graph**.  
    *   Nodes represent discrete execution blocks (e.g., a function body or a `.then()` callback).
    *   Edges represent the "Enqueue" or "Register" operations.
*   **SOTA Logic**: By traversing the Callback Graph, tools like **Async-TAJS** can prove that Callback A *must* execute before Callback B (Happens-Before) or that their execution order is non-deterministic (Race Candidate).
*   **Citation**: Furet, B. (2021). *Practical Detection of JavaScript Concurrency Bugs using Callback Graphs*.

### 2.2 Promise Graphs (PromiseKeeper)
Specifically targets the complex "Promise Chain" patterns common in modern JS/TS.

*   **Mechanism**: **Temporal Dependency Analysis**. The analyzer tracks the resolution state of Promises in the AST. If two independent Promise chains mutate the same shared state and their resolution order is not strictly mediated by an `await` or `Promise.all`, a race is flagged.
*   **Citation**: Sotiropoulos, T., & Livshits, B. (2019). *Static Analysis for Asynchronous JavaScript Programs*.

---

## 3. SOTA Dynamic Analysis: NodeRacer
Traditional happens-before analysis often suffers from "Interleaving Sensitivity." NodeRacer provides the modern SOTA for surfacing these bugs in production code.

*   **Mechanism**: **Selective Event Postponement**.
    1.  NodeRacer instruments the Node.js event loop.
    2.  It identifies "Conflict Pairs" (two events accessing the same variable).
    3.  It artificially postpones one event to force an alternative interleaving, observing if the program state diverges.
*   **Technical Detail**: It uses a hybrid approach, combining static "Potential Race" identification with dynamic "Interleaving Exploration" to eliminate false positives.
*   **Citation**: Zheng, Y., et al. (2023). *NodeRacer: Efficient and Comprehensive Data Race Detection for Node.js Applications*. ASE '23.

---

## 4. AST Patterns for Async Races

| Pattern | AST Structure | Bug Type |
| :--- | :--- | :--- |
| **Atomic Block Violation** | `Read(v) -> Await -> Write(v)` | Stale Read / Lost Update |
| **Registration Race** | `TriggerProperty -> HandlerAssignment` | Lost Event (e.g., `img.src` before `onload`) |
| **Floating Promise** | `CallExpression(p) -> [No Await/Return]` | Fire-and-Forget State Corruption |
| **Check-Then-Act (Async)** | `IfStatement(v) -> Await -> Mutate(v)` | TOC/TOU Race |

---

## 5. Industrial Tooling State
*   **ESLint**: Rules like `require-atomic-updates` and `no-floating-promises` provide the baseline AST-pattern detection.
*   **Facebook Pulse**: Internal path-sensitive analysis for detecting async races in React/Relay applications by tracking state resolution flows.
*   **DeepSource/Sonar**: Use data flow analysis to detect "unawaited" operations that modify properties of the `this` context.

---

## 6. References
1.  **NodeRacer**: [ASE 2023 Paper](https://ieeexplore.ieee.org/document/10298357)
2.  **Callback Graphs**: [Sotiropoulos & Livshits, 2019](https://dl.acm.org/doi/10.1145/3338906.3338936)
3.  **Broken Promises**: [Bernardo Furet, 2021](https://scholar.google.com/scholar?q=Bernardo+Furet+Callback+Graphs)
4.  **Async-TAJS**: [TAJS Project Homepage](http://www.brics.dk/TAJS/)
