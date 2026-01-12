# SOTA Report: Deserialization & Prototype Pollution (Security-03)

**Date**: January 2026  
**Subject**: Analysis of global object pollution in JavaScript and automated gadget chain discovery in Java/.NET.

## 1. Executive Summary
Modern web security is increasingly threatened by vulnerabilities that manipulate application state or execution flow through serialized data. State-of-the-Art (SOTA) research focus has shifted from finding "The Sink" to finding the **Gadget Chain**—the sequence of benign method calls that, when triggered by untrusted data, result in Remote Code Execution (RCE).

---

## 2. Prototype Pollution (JavaScript SOTA)
Prototype Pollution involves injecting properties into `Object.prototype`, which are then inherited by all objects.

*   **SOTA Detection**: **Object Dependence Graphs (ODG)**.
    *   Unlike linear taint tracking, ODG models the prototype chain as a graph of dependencies.
    *   Tools like **ObjLupAnsys** use flow- and context-sensitive analysis to track if a property lookup (e.g., `obj.isAdmin`) can be influenced by a polluted prototype.
*   **The "Gadget" Focus**: Recent research (e.g., **PROBETHEPROTO**) focuses on identifying "Gadgets"—code patterns that use properties in dangerous ways (like `eval(polluted_prop)` or setting `template_path`).

---

## 3. Insecure Deserialization (Java/.NET/Python)
Deserialization vulnerabilities occur when an application restores an object from a stream without validating the underlying class.

### 3.1 Gadget Chain Mining
SOTA tools now automate the discovery of RCE paths.

*   **JDD (Java Deserialization Detector)**: A bottom-up static analysis tool that starts from dangerous "Sinks" (e.g., `Runtime.exec`) and works backward to find "Gadget Fragments" that can be reached from a `readObject()` call.
*   **FLASH**: Uses **Deserialization-guided Call Graph Construction**. It handles Polymorphism and Dynamic Dispatch (e.g., `interface.method()`) more precisely than standard static analysis, reducing false negatives.
*   **GCMiner**: Integrates dynamic features (Fuzzing) to verify that a statically-found chain is actually exploitable by generating valid serialized payloads.

---

## 4. Automated Mitigation: QUACK (PHP)
While detection is key, SOTA research also explores "Auto-Healing" for legacy apps.

*   **QUACK Mechanism**: Statically analyzes PHP code to identify which classes are actually expected to be deserialized.
*   **Action**: It automatically refactors the code to implement a **Strict Allow-list** for the `unserialize()` function, neutralizing unknown gadget chains without developer intervention.

---

## 5. Comparison of Discovery Techniques

| Tool | Language | Precision Strategy | Advantage |
| :--- | :--- | :--- | :--- |
| **CodeQL** | Multi | Logic-based QL Queries | Industry-standard, low FP |
| **GadgetInspector**| Java | Static Taint Tracking | Breadth of search |
| **JDD** | Java | Bottom-up Verification | Zero-day discovery |
| **ODGen** | JS | Object Dependence Graph | Handles Prototype Chain |

---

## 6. References
1.  **CodeQL Prototype Pollution**: [GitHub Security Research](https://github.com/github/codeql/blob/main/javascript/ql/src/Security/CWE-1321/PrototypePollution.ql)
2.  **JDD**: [ResearchGate: Automated Deserialization Gadget Discovery](https://www.researchgate.net/publication/351656201_JDD_A_Framework_for_Detecting_Java_Deserialization_Vulnerabilities)
3.  **FLASH**: [USENIX '24: Precisely Mining Gadget Chains](https://www.usenix.org/conference/usenixsecurity24/presentation/flash)
4.  **QUACK**: [Static Mitigation of PHP Deserialization](https://www.cs.brown.edu/~vjk/QUACK_Paper.pdf)
5.  **ObjLupAnsys**: [Politecnico di Milano: Analyzing Prototype Pollution](https://www.polimi.it/en/scientific-research/research-publications)
