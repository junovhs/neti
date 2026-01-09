# SOTA Report: Return Value Semantics vs Naming (Semantic-04)

**Date**: January 2026  
**Subject**: Intent alignment, nullable return types, CodeBERT prediction, and side-effecting getters.

## 1. Executive Summary
Return value semantics define how a function's output is handled (by value, reference, or pointer) and what it implies (success, error, or state). State-of-the-Art (SOTA) research focuses on **Neural Type Prediction** and **Misalignment Detection**, where the function's name and its return behavior diverge, leading to "False Intent" bugs.

---

## 2. Type-Intent Alignment
A primary SOTA goal is ensuring that the "Name" promises what the "Type" delivers.

*   **Ambiguity Example**: A function named `getUser()` that returns `void` (implying it updates a global user) vs. one that returns a `User` object.
*   **Semantic Nuance**: SOTA practices recommend explicit naming for return mechanisms (e.g., `elementValueAt` vs `elementRefAt`). Static analysis tools (DeepSource, Sonar) now flag "Inconsistent Return Statements" where a function returns a value in some paths but not others.

---

## 3. CodeBERT: Return Type Prediction
Microsoft's **CodeBERT** has achieved SOTA results in predicting return types from function names and docstrings.

*   **Accuracy**: SOTA research shows >94% accuracy in identifying the intended return type from API documentation alone.
*   **Semantic Drift**: If a function is named `fetchData` but returns a `Boolean`, CodeBERT can flag this as a semantic mismatch, suggesting that the name implies a data container return.

---

## 4. Misleading Returns & Side-Effects
The most dangerous semantic misalignment is the "Side-effecting Getter."

*   **Anti-pattern**: A method named `calculateTotal()` that also increments a counter or writes to a database.
*   **Detection**: SOTA semantic analyzers use NLP to identify "Verb-Noun" mismatches. If a "Getter" (noun-focused) contains "Update" logic (verb-focused), it is flagged.
*   **JavaScript SOTA**: DeepSource (JS-0037) flags returning values from "Setter" methods, as these values are typically unreachable and indicate logic confusion.

---

## 5. Summary of Return Semantics SOTA

| Issue | Detection Logic | SOTA Tool/Model |
| :--- | :--- | :--- |
| **Inconsistent Return** | Path-based type checking | DeepSource / Pylance |
| **Name-Type Mismatch** | Predicted (CodeBERT) != Actual | CodeBERT / GraphCodeBERT |
| **Side-effecting Getter**| NLP "Update" detection in Getter | Semantic Linter |
| **Nullable Drift** | Nullability flow analysis | Pyright / Strict C# |

---

## 6. References
1.  **Microsoft Research**: [CodeBERT: Bimodal Pre-training for NL-PL](https://arxiv.org/abs/2002.08155)
2.  **DeepSource**: [Style and Semantic Analysis of Return Statements](https://deepsource.io/directory/analyzers/python/style/)
3.  **SonarSource**: [RSPEC-1172: Unused Parameter and Return Semantics](https://rules.sonarsource.com/)
4.  **Uri Alon**: [code2vec: Predicting Method Names and Returns](https://code2vec.org/)
5.  **vFunction**: [Modern Semantic Complexity in Microservices](https://vfunction.com/)
