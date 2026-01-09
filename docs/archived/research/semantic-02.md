# SOTA Report: Multi-dimensional Complexity (Semantic-02)

**Date**: January 2026  
**Subject**: Beyond McCabe, CRAP score, Information Flow, and Data-flow analysis.

## 1. Executive Summary
Traditional Cyclomatic Complexity (McCabe) only counts decision paths. State-of-the-Art (SOTA) research recognizes that true complexity is multi-dimensional, involving **Human Cognition**, **Structural Coupling**, and **Test Sufficiency**. Modern metrics like **C.R.A.P.** and **Cognitive Complexity** provide a more balanced view of technical debt and maintenance risk.

---

## 2. The C.R.A.P. Metric (Change Risk Anti-Pattern)
One of the most practical SOTA metrics, C.R.A.P. combines complexity with code coverage.

*   **Formula**: $CRAP(m) = C(m)^2 \times (1 - Cov(m))^{3} + C(m)$
    *   Where $C(m)$ is Cyclomatic Complexity and $Cov(m)$ is % code coverage.
*   **Predictive Power**: It identifies code that is "Complex but Untested." Such code is statistically the most likely to contain bugs and the most dangerous to refactor.

---

## 3. Cognitive Complexity (Understandability SOTA)
Introduced by SonarSource, this metric aims to reflect the actual mental load of a developer.

*   **Key Heuristics**:
    *   **Nesting Penalty**: Deeply nested `if` statements are penalized more heavily than sequential ones.
    *   **Logic Breaks**: Shorthand notation (e.g., ternary operators) is treated as less complex than full `if/else` blocks if they don't break the cognitive flow.
*   **Validation**: Large-scale studies show it correlates more strongly with the time it takes a developer to fix a bug than lines of code (LOC).

---

## 4. Information Flow (Structural SOTA)
Based on the work of **Henry and Selig**, this evaluates how data "fans" through a module.

*   **Metrics**:
    *   **Fan-in**: Number of local flows into a procedure.
    *   **Fan-out**: Number of local flows out of a procedure.
*   **SOTA Logic**: `Complexity = (Fan-in * Fan-out)^2`. This captures the complexity of "The Glue" in a system, identifying components that are overloaded with architectural responsibility.

---

## 5. Summary of Multi-dimensional Models

| Metric | Dimension | Optimization Goal |
| :--- | :--- | :--- |
| **C.R.A.P.** | Complexity vs. Coverage | Reduce high-risk / untested paths |
| **Cognitive** | Human Readability | Reduce nesting / simplify logic |
| **SFOUT** | Structural Coupling | Decouple heavily-dependent modules |
| **DFA Lattice**| Data States | Identify unreachable / stale data |
| **Maintainability Index**| Composite Health | Keep above 20 for longevity |

---

## 6. References
1.  **SonarSource**: [Cognitive Complexity Specification](https://www.sonarsource.com/docs/CognitiveComplexity.pdf)
2.  **NDepend**: [The C.R.A.P. Metric Explained](https://www.ndepend.com/metrics#CRAP)
3.  **Henry & Selig**: (2020). *Hybrid Information-Flow Metrics for System Complexity*.
4.  **vFunction**: [Modern Microservices Complexity Analysis](https://vfunction.com/blog/calculating-software-complexity/)
5.  **DORA**: [Predicting Deployment Pain with System Complexity](https://dora.dev/)
