# SOTA Report: Dead Parameters (Unused) (Semantic-03)

**Date**: January 2026  
**Subject**: Inter-procedural liveness, DCE-LLM, and safe signature refactoring.

## 1. Executive Summary
"Dead Parameters" are method arguments that are declared but never consumed by any logic sink. While often considered a minor "code smell," State-of-the-Art (SOTA) research reveals that dead parameters are often remnants of **Stale Architecture** or **Incomplete Features**. SOTA tools move beyond single-function linting to **Inter-procedural Liveness Analysis**—tracking data "cargo" that is passed through layers without usage.

---

## 2. Inter-procedural Liveness (IPO SOTA)
Detecting dead parameters at scale requires analyzing the entire program's Call Graph.

*   **Mechanism**: **Iterative Fixed-Point Analysis**.
    *   The analyzer tracks "Live-Out" variables across function boundaries.
    *   If a parameter $P$ in function $F$ is only used to call function $G(P)$, and $G$ doesn't use $P$, then $P$ is flagged as "Inter-procedurally Dead" in both $F$ and $G$.
*   **Scalability**: SOTA research (VMCAI'23) uses **Graph Sparsity** (treedepth/treewidth) to speed up these queries, allowing analysis of codebases with >500k lines of code in constant-time locally.

---

## 3. DCE-LLM (Automated Patching)
A 2024/2025 SOTA trend is using Small Language Models (SLMs) to clean up dead code.

*   **Logic**:
    *   **Attribution-based Selection**: A model (like CodeBERT) identifies "suspect" unused lines.
    *   **LLM Judgment**: An LLM validates if the parameter is truly dead or required by an interface/protocol.
    *   **Automated Patching**: The LLM generates a safe refactoring patch (including call-site updates) with high F1 accuracy.

---

## 4. Language Idioms and Suppression
Modern languages have formalized the "Intentional Unused" pattern.

*   **Rust**: Prefixing with `_` (e.g., `_param`) is a compiler-level directive.
*   **C++17/C23**: `[[maybe_unused]]` or unnamed parameters provide standard-compliant suppression.
*   **Go**: The blank identifier `_` is used for explicit discarding in assignments, though parameter discarding usually involves naming conventions.

---

## 5. Summary of Dead Parameter Detection

| Feature | Legacy Approach | SOTA (2025/2026) |
| :--- | :--- | :--- |
| **Scope** | Single function lint | Whole-Program IPO |
| **Logic** | Syntactic check | Data Flow Fixed-Point |
| **Refactoring** | Manual deletion | Type-safe Refactoring Engines |
| **Validation** | Developer Guess | SAFEREFACTOR (Automated Testing) |

---

## 6. References
1.  **DCE-LLM**: (2024). *DCE-LLM: LLM-Based Dead Code Elimination Framework*. [arXiv:2410.12351](https://arxiv.org/abs/2410.12351)
2.  **Henry & Selig**: (2020). *Inter-procedural Optimization and Liveness*.
3.  **VMCAI'23**: *Exploiting Graph Sparsity for Efficient Data-Flow*.
4.  **JetBrains**: [IntelliJ IDEA: Declaration Redundancy Inspection](https://www.jetbrains.com/help/idea/inspections-declaration-redundancy.html)
5.  **SVF-lib**: [Scalable Value-Flow Analysis for LLVM](https://svf-tools.github.io/SVF/)
