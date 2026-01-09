# SOTA Report: Auto-Refactoring Global State (State-04)

**Date**: January 2026  
**Subject**: Algorithmic techniques for automatically transforming global state into Dependency Injection (DI) and encapsulated modules.

## 1. Executive Summary
Automatically refactoring global state is not merely a string-replacement task; it requires identifying the "Optimal Injection Point" using call-graph theory and verifying behavioral preservation using dynamic invariant detection. State-of-the-Art (SOTA) pipelines combine **Lowest Common Ancestor (LCA)** algorithms with **Daikon-based Invariant Verification** to safely migrate legacy systems to modern Dependency Injection architectures.

---

## 2. The Injection Point Algorithm: Lowest Common Ancestor (LCA)
The primary challenge in DI refactoring is deciding *where* in the call hierarchy to inject the formerly global dependency.

*   **Mechanism**: **Call Graph LCA Analysis**.
    1.  Construct a directed call graph \( G \) where nodes are functions and edges represent calls.
    2.  Identify all leaf nodes \( \{L_1, L_2, ... L_n\} \) that directly access the global variable \( V \).
    3.  Compute the **Lowest Common Ancestor (LCA)** of the set \( \{L_i\} \) in the call graph.
    4.  **SOTA Rule**: The LCA node represents the most granular point in the architecture where the dependency can be "owned" and subsequently passed (injected) to its children.
*   **Significance**: This prevents "Dependency Bloat" (passing the object through every function in the system) by localizing the injection to the specific execution sub-tree that requires it.

---

## 3. Correctness Verification: Dynamic Invariants (Daikon)
Static analysis can identify *where* to change, but dynamic analysis is required to ensure *behavioral parity*.

*   **Tool**: **Daikon Invariant Detector**.
*   **Process**:
    1.  **Pre-Refactoring**: Run the system under test and use Daikon to infer "likely invariants" (e.g., `global_v > 0`, `global_v` is sorted).
    2.  **Refactoring**: Apply the AST transformation (Encapsulation/Injection).
    3.  **Post-Refactoring**: Re-run the tests on the refactored code. Use Daikon to verify that the invariants of the now-injected state still hold.
*   **Citation**: Kataoka, Y., et al. (2001). *Automated Support for Program Refactoring using Invariants*. ICSM '01.

---

## 4. Structural Refactoring by Abstraction
The formal model for moving from concrete global access to abstract interface dependency.

*   **Mechanism**: **Interface Extraction & Propagation**.
    1.  **Encapsulation**: Wrap the global variable in a Class/Struct with private accessors.
    2.  **Interface Extraction**: Generate an Interface reflecting the usage patterns (e.g., `IGlobalState`).
    3.  **Constructor Propagation**: Use AST rewriting to update all constructors from the LCA down to the leaf nodes to accept `IGlobalState`.
*   **SOTA Insight**: Modern refactoring engines (e.g., **RefactoringMiner**, **DepRefactor**) utilize **Points-to Analysis** to identify clusters of global state that are always accessed together, suggesting they should be refactored into a single DTO (Data Transfer Object) rather than individual parameters.

---

## 5. Synthesis: Refactoring Pipeline

| Phase | Technique | SOTA Tool/Algorithm |
| :--- | :--- | :--- |
| **Discovery** | Points-to Analysis | LLVM / Soot |
| **Localization** | Call Graph LCA | Tree-Traversal Algorithms |
| **Transformation** | AST Rewriting | LibTooling / Spoon |
| **Validation** | Invariant Matching | Daikon / JPF |

---

## 6. References
1.  **Refactoring with Invariants**: [Kataoka et al., 2001](https://homes.cs.washington.edu/~mernst/pubs/invariants-refactoring-icsm2001.pdf)
2.  **LCA in Call Graphs**: Conceptual framework for DI optimization.
3.  **Daikon**: [The Daikon Invariant Detector Official Docs](https://plse.cs.washington.edu/daikon/)
4.  **RefactoringMiner**: [SOTA Tool for Refactoring Detection](https://github.com/tsantalis/RefactoringMiner)
