# SOTA Report: Algorithmic Complexity AST Patterns (Performance-01)

**Date**: January 2026  
**Subject**: Static detection of O(n^2) patterns, Worst-Case Execution Time (WCET), and cost analysis.

## 1. Executive Summary
Automatically determining Big-O complexity from source code is an undecidable problem in the general case (Halting Problem). However, State-of-the-Art (SOTA) tools effectively use **Symbolic Execution** and **Heuristic Pattern Matching** to provide upper bounds for "Performance Bugs." Tools like **Infer Cost** and **AiT** lead the field by identifying nested loop structures and solving constraints on loop-invariant variables.

---

## 2. Infer Cost (Asymptotic SOTA)
Facebook's Infer uses a tiered approach to calculate procedure-level costs.

*   **Phase 1: InferBo (Interval Analysis)**: Computes value ranges for variables.
*   **Phase 2: Loop Bound Inference**: Identifies loop exit conditions and determines how many times the loop body can be reached.
*   **Phase 3: Constraint Solving**: Generates a polynomial representing the "Cost" of a function.
*   **Success**: It identifies "Complexity Regressions"—code changes that increase complexity (e.g., O(n) becoming O(n^2)) before deployment.

---

## 3. WCET for Embedded Systems
In safety-critical systems (Aerospace/Automotive), "Approximate" isn't enough. We need a guaranteed maximum.

*   **AiT (AbsInt)**: The industry standard. It analyzes binary executables and uses formal hardware models to account for **Cache Misses**, **Pipeline Stalls**, and **Branch Prediction**.
*   **Hybrid Analysis (RapiTime)**: Combines static structure analysis with real-time hardware measurements to provide a more precise (less pessimistic) bound than pure static analysis.

---

## 4. Annotation-Based SOTA
A growing trend is "Complexity by Contract."

*   **The Paradigm**: Developers annotate functions with `@Complexity("O(n^2)")`. 
*   **Static Verification**: The analyzer checks if the implementation matches the annotation. If a nested loop is added where `@Complexity("O(n)")` was promised, the build fails.
*   **Citation**: (2020). *Annotation-Based Static Verification of Algorithmic Complexity in Java*.

---

## 5. Summary of SOTA Detection Patterns

| Pattern | Complexity Proxy | SOTA Indicator |
| :--- | :--- | :--- |
| **Nested Loop** | O(n^d) | Loops where both bounds relate to the same `Collection.size()` |
| **Recursive Call** | Expo/N-Log | Paths that return result from `f(n-1) + f(n-2)` without memoization |
| **Cross-Product** | O(n*m) | Iterating over `Table A` inside a loop over `Table B` |
| **Monotonic Buffer**| Memory O(n) | `list.add()` inside a loop without a corresponding `clear()` or size cap |

---

## 6. References
1.  **Infer Cost Documentation**: [Static Complexity Analysis](https://fbinfer.com/docs/checker-cost)
2.  **AiT WCET**: [AbsInt: Worst-Case Execution Time Analysis](https://www.absint.com/ait/)
3.  **COAL**: (2021). *Complexity Analysis for Java Collections*.
4.  **RapiTime**: [Rapita Systems: Hybrid Timing Analysis](https://www.rapitasystems.com/products/rapitime)
5.  **Annotation-Based Complexity**: [CEUR Workshop Proceedings: Java Complexity Verification](https://ceur-ws.org/Vol-2722/paper5.pdf)
