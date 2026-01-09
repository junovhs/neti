# SOTA Report: Unbounded Collection Growth (Resource-02)

**Date**: January 2026  
**Subject**: Analysis of memory accumulation in shared data structures, static size inference, and dynamic heap-dominator analysis.

## 1. Executive Summary
Unbounded collection growth (often called "Logical Memory Leaks") occurs when objects are correctly referenced by the language runtime but are functionally irrelevant to the application. State-of-the-Art (SOTA) detection combines **Dominator Tree Analysis** with **Object Age Statistics** to identify accumulation points where the "Retained Size" of a collection grows non-linearly with respect to time.

---

## 2. Structural Analysis: The Dominator Tree
The Dominator Tree is the SOTA representation for visualizing heap ownership.

*   **Mechanism**: Object A *dominates* Object B if every path from the GC roots to B passes through A.
*   **SOTA Logic**: By calculating the **Retained Size** (the total memory freed if an object is collected), tools identify "Memory Hotspots."
*   **The "Big Drop" Indicator**: A structural anomaly where a small parent object (e.g., a `List`) dominates a massive amount of memory (e.g., millions of small `String` instances). 
*   **Citation**: Lengauer, T., & Tarjan, R. (1979). *A Fast Algorithm for Finding Dominators in a Flowgraph*.

---

## 3. Dynamic Predictors: Object Age and Staleness
Modern research moves beyond "is it leaking?" to "will it leak?".

### 3.1 Object Staleness (Leakbot)
Staleness measures the time since an object was last accessed versus its current age.

*   **SOTA Metric**: **Staleness-to-Age Ratio**. If a collection's objects have high staleness while the collection itself continues to grow, it is a high-confidence indicator of an unbounded leak.
*   **Tooling**: **IBM Leakbot** uses temporal snapshots to track how leak candidates evolve over time.
*   **Citation**: (2019). *Machine Learning-Based Prediction of Memory Leaks in Java Apps*.

### 3.2 Generational Survivorship
*   **SOTA Logic**: Objects that survive multiple "Major GCs" but reside in a collection that never shrinks are flagged as "Zombie State."

---

## 4. Static Analysis: Abstract Interpretation
Static analysis attempts to prove the "Bound" of a collection size.

*   **Mechanism**: **Widening Operators**.
    *   In Interval Abstraction, indices and sizes are tracked. 
    *   If a loop contains a `.push()` or `.add()` and the analyzer cannot find a corresponding `.clear()` or conditional exit, the widening operator approximates the size to `[0, ∞]`.
*   **Constraint**: Sized types and symbolic execution are used to find "Monotonic Size Increment" paths (AST branches where the collection size only increases).

---

## 5. Summary of SOTA Analysis Metrics

| Metric | Grain | Goal |
| :--- | :--- | :--- |
| **Retained Size** | Heap-Based | Quantifying Leak Impact |
| **Dominator Depth** | Graph-Based | Identifying Root Ownership |
| **Object Staleness** | Temporal | Proactive Leak Prediction |
| **Size Widening** | Static / Abstract | Formal Boundary Verification |

---

## 6. References
1.  **Lengauer-Tarjan**: [Original Algorithm Paper](https://dl.acm.org/doi/10.1145/357062.357071)
2.  **Leakbot Research**: [IBM Research: Autonomous Tool for Leak Detection](https://dl.acm.org/doi/10.1145/1101908.1101962)
3.  **Eclipse MAT**: [SOTA Tool for Dominator Analysis](https://www.eclipse.org/mat/)
4.  **Widening Operators**: [Cousot & Cousot: Abstract Interpretation Basics](https://en.wikipedia.org/wiki/Widening_(computer_science))
