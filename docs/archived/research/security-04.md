# SOTA Report: Sink-Reaching Heuristics (Security-04)

**Date**: January 2026  
**Subject**: Lightweight security analysis, approximate taint tracking, and AST-based pattern matching at scale.

## 1. Executive Summary
While full-program taint analysis is mathematically sound, it is often too slow for CI/CD pipelines or massive monorepos. State-of-the-Art (SOTA) research has introduced **Approximate Taint Analysis** and **AST Search Patterns** as lightweight proxies. These tools trade theoretical soundness for developer velocity, focusing on "Reachable Member Analysis" to flag suspicious code without prove-it-all data flow.

---

## 2. Taint Mode vs. Search Mode (The Semgrep Shift)
SOTA tools like **Semgrep** categorize analysis into two tiers:

*   **Search Mode (Heuristic)**: Matches AST structures (e.g., "Any call to `exec()` where the argument isn't a hardcoded string").
    *   **Advantage**: Instantaneous (ms), works locally on individual files.
*   **Taint Mode (Formal)**: Tracks data flow from `sources` to `sinks` across function boundaries.
    *   **Advantage**: Higher precision, lower false positives for complex logic.
*   **SOTA Paradigm**: Use Search Mode for "Massive-Scale Surface Scanning" to identify broad risks, then trigger Taint Mode for "Deep Validation."

---

## 3. Approximate Taint: Coarse-Grained Logic
In environments like Android (10k+ files), precise analysis often fails due to state explosion.

*   **SOTA Logic**: **Coarse-Grained Propagation**. Instead of tracking individual variables, the analyzer marks an entire scope or object as "Potentially Tainted" if any part of it is influenced by a source.
*   **Modular Analysis**: Analyzing functions in isolation and generating a "Taint Summary" for the function (e.g., "This function returns a tainted string if parameter A is tainted"). This allows for incremental scanning of large changesets.

---

## 4. Bimodal Taint Analysis (ML + Static)
Recent papers explore **Bimodal Analysis** to bridge the gap between heuristics and formal proof.

*   **Mechanism**: A machine learning model identifies "Unexpected Flows"—data paths that don't match typical application logic.
*   **Static Hook**: The static analyzer then focuses its expensive data flow checks *only* on these high-probability areas, rather than the entire codebase.
*   **Citation**: (2021). *Bimodal Taint Analysis: Leveraging ML for Security Focus*.

---

## 5. Summary of Analysis Trade-offs

| Method | Latency | Soundness | Best Use Case |
| :--- | :--- | :--- | :--- |
| **Formal Taint** | High (mins/hours) | High | Critical Security Audits |
| **AST Search** | Low (millis) | Low | Pre-commit / IDE Linter |
| **Modular Taint** | Medium (seconds) | Medium | CI/CD PR Scanning |
| **Type-Based** | Low (millis) | High* | Typed Langs (Rust/Haskell) |

---

## 6. References
1.  **Semgrep Taint vs Search**: [Semgrep Documentation: Data Flow Analysis](https://semgrep.dev/docs/writing-rules/data-flow/taint-mode/)
2.  **SonarQube Heuristics**: [Clean Code Security: Tracking Sinks](https://www.sonarsource.com/solutions/security/)
3.  **Modular Static Analysis**: [Amazon Science: Scalable Taint for Large Systems](https://www.amazon.science/publications/making-static-analysis-modular-and-incremental)
4.  **Bimodal Analysis**: [arXiv: Probabilistic Security Flows](https://arxiv.org/abs/2105.02105)
5.  **Type-Based Taint**: (2023). *Modular and Incremental Taint Checking*.
