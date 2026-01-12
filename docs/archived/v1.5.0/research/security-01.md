# SOTA Report: Taint Analysis for Dynamic Languages (Security-01)

**Date**: January 2026  
**Subject**: Challenges and State-of-the-Art (SOTA) techniques for tracking data flow in JavaScript, Python, and Ruby.

## 1. Executive Summary
Taint analysis in dynamic languages is complicated by features like `eval()`, prototype pollution, and asynchronous execution. SOTA solutions have moved beyond simple string-matching to **Hybrid Taint Engines** that combine static **Abstract Interpretation** with high-performance **Dynamic Taint Analysis (DTA)**.

---

## 2. Dynamic Language Challenges
*   **Dynamic Property Access**: `obj[x]` where `x` is user-controlled makes static tracking of object properties nearly impossible without sophisticated aliasing analysis.
*   **Context Loss in Async**: Taint must be propagated through `Promises` and `async/await` boundaries.
*   **Vague Type Info**: Without static types, flow through complex data structures (Maps, Sets) often results in "Under-tainting."

---

## 3. Tooling SOTA: Augur and TruffleTaint

### 3.1 Augur (Async-Aware DTA)
Designed for modern ES7 JavaScript.

*   **Mechanism**: **VM-Supported Instrumentation**. Augur hooks into the JS virtual machine's event loop to track taint markers across `await` suspension points.
*   **SOTA Advantage**: It maintains taint context through the microtask queue, which traditional source-to-source instrumenters often lose.
*   **Citation**: (2020). *Augur: High-Performance Dynamic Taint Analysis for Modern JavaScript*.

### 3.2 TruffleTaint (GraalVM Multi-Language)
Targets applications that mix languages (e.g., Python calling C++ extensions).

*   **Mechanism**: **Universal Intermediate Representation (IR)**. All languages in the GraalVM ecosystem share an IR. TruffleTaint propagates taint markers through this IR.
*   **Result**: Taint can flow from a Python `input()` through a C++ parser and into a Java `PreparedStatement` without losing visibility.
*   **Citation**: (2021). *Multi-Language Dynamic Taint Analysis*.

---

## 4. Specification Inference: InspectJS
A major SOTA shift is using Machine Learning to define "What is a Sink?".

*   **The Problem**: New libraries appear daily; manual lists of source/sink APIs are always outdated.
*   **SOTA Solution**: **InspectJS** uses ML models trained on millions of GitHub commits to automatically infer if a function call (e.g., `db.run()`) acts as a sensitive sink based on its call-site context and parameter naming.
*   **Citation**: (2022). *InspectJS: Inferring Taint Specifications for JavaScript Libraries*.

---

## 5. Summary of Analysis Techniques

| Technique | Grain | Pros | Cons |
| :--- | :--- | :--- | :--- |
| **Static Taint (CodeQL)** | Global | High coverage, zero runtime cost | Higher False Positives |
| **Dynamic Taint (DTA)** | Runtime | High precision, low False Positives | High runtime overhead |
| **ML-Inferred Sinks** | Statistical | Scans new/unknown libraries | Non-deterministic |
| **Abstract Interpretation** | Formal | Provable security properties | High complexity |

---

## 6. References
1.  **Augur**: [ResearchGate: High-Performance JS Taint](https://www.researchgate.net/publication/343213501_Augur_High-Performance_Dynamic_Taint_Analysis_for_Modern_JavaScript)
2.  **TruffleTaint**: [arXiv: Multi-Language Flow](https://arxiv.org/abs/2103.14961)
3.  **InspectJS**: [Microsoft Research: ML for Taint Specs](https://www.microsoft.com/en-us/research/publication/inspectjs-inferring-taint-specifications-for-javascript-libraries/)
4.  **CodeQL Taint**: [GitHub CodeQL Documentation](https://codeql.github.com/docs/codeql-language-guides/using-flow-summary-for-javascript-library-modeling/)
