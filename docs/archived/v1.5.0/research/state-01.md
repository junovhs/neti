# SOTA Report: Deep Mutability Detection (State-01)

**Date**: January 2026  
**Subject**: Static analysis techniques for detecting transitive object mutation (deep mutability) without full data flow analysis.

## 1. Executive Summary
Traditional deep mutability detection often relies on full-program data flow analysis (DFA), which is computationally expensive and difficult to scale. State-of-the-Art (SOTA) research over the last two decades has pivoted towards **Type-Based Reference Immutability** and **Ownership Systems**. These mechanisms provide modular, compile-time guarantees by treating mutability as a first-class citizen of the type system, often eliminating the need for iterative DFA while maintaining precision.

---

## 2. Core Mechanisms & Academic Foundations

### 2.1 Reference Immutability (Javari)
Javari introduced a backward-compatible extension to Java that allows for transitive immutability via type qualifiers.

*   **Mechanism**: Uses the `readonly` qualifier. If a reference is marked `readonly`, the type system prevents modifications to any field of the object and any state reachability from it (transitive state).
*   **DFA Shortcut**: Instead of tracking every potential alias, Javari enforces rules at every field access. If `x` is `readonly`, then `x.f` is automatically treated as `readonly`, regardless of its declared type.
*   **Citation**: Tschantz, M. S., & Ernst, M. D. (2005). *Javari: Adding Reference Immutability to Java*. OOPSLA '05. [OOPSLA 2005: 211-230].

### 2.2 Transitive Class Immutability (Glacier)
Glacier focuses on the "immutability by default" paradigm for classes rather than references.

*   **Mechanism**: Uses the `@Immutable` annotation on classes. The checker ensures that all fields of an `@Immutable` class are also `@Immutable`. 
*   **DFA Shortcut**: Enforcement is modular. The checker only needs to inspect the class declaration and its immediate field types, ensuring the chain of immutability is never broken by a mutable field.
*   **Citation**: Van Atta, M. D., et al. (2017). *Glacier: A Language Research Tool for Transitive Class Immutability*. ICSE '17.

### 2.3 Ownership & Immutability (OIGJ)
OIGJ (Ownership and Immutability Generic Java) unifies ownership (who owns the object) and immutability (who can change it).

*   **Mechanism**: Uses **Viewpoint Adaptation**. When accessing a field `f` of an object `o`, the mutability of the resulting reference is calculated based on the mutability of `o` *and* the declared mutability of `v` at that site.
*   **Technical Detail**: It distinguishes between `Mutable`, `Immutable` (no one can change it), and `ReadOnly` (this reference cannot change it).
*   **Citation**: Zibin, Y., et al. (2010). *Object and Reference Immutability using Ownership*. OOPSLA '10.

---

## 3. Industrial Implementations & Language-Level Integration

### 3.1 Rust: Borrow Checking & Interior Mutability
Rust provides the most widespread industrial application of these concepts.

*   **Mechanism**: The **Exclusivity Rule** (`&mut T` vs `&T`). Deep mutability is handled by the fact that a `&T` reference is transitively immutable by default.
*   **SOTA Aspect**: To allow controlled mutation, Rust uses "Interior Mutability" wrappers (`RefCell`, `Mutex`). Static analysis (the borrow checker) ensures that deep immutability is only violated under strictly defined safety boundaries.
*   **Citation**: Matsakis, N. D., & Klock II, F. S. (2014). *The Rust Language*. Ada User Journal.

### 3.2 Pony: Reference Capabilities
Pony uses an actor-model with a sophisticated capability-based type system.

*   **Mechanism**: **Reference Capabilities** (`val`, `ref`, `iso`). 
    *   `val` represents a transitively immutable reference that can be safely shared across actors.
    *   `iso` represents an isolated, mutable reference that can be "sent" to another actor, losing access in the sender to prevent races.
*   **DFA Shortcut**: The type system tracks "aliases" and "capabilities" at the call site, allowing the compiler to prove safety without global data flow.
*   **Citation**: Clebsch, S., et al. (2013). *Reference Capabilities for Safe Parallelism*. OOPSLA '11/13.

---

## 4. Synthesis of SOTA Principles

| Technique | Grain | Granularity | Key Restriction |
| :--- | :--- | :--- | :--- |
| **Reference Immutability** | Per-Reference | Deep/Transitive | Cannot mutate via *this* path |
| **Class Immutability** | Per-Type | Shallow/Deep | All instances are immutable |
| **Ownership Types** | Per-Context | Deep/Transitive | Encapsulation of fields |
| **Ref Capabilities** | Per-Alias | Deep/Transitive | Control over sharing/mutability |

### SOTA Insight: Beyond Basic Linting
Modern static analysis tools (like the **Checker Framework** for Java) have integrated these OOPSLA-grade theories into real-world compilers. They provide "Flow-Sensitive Type Inference," which is a lightweight version of DFA that only operates locally within method boundaries, relying on the robust type-qualifier backbone for global correctness.

---

## 5. References & Further Reading
1.  **Javari**: [OOPSLA 2005 Paper](https://homes.cs.washington.edu/~mernst/pubs/javari-oopsla2005.pdf)
2.  **OIGJ**: [OOPSLA 2010 Paper](https://homes.cs.washington.edu/~mernst/pubs/oigj-oopsla2010.pdf)
3.  **checker-framework**: [Official Documentation](https://checkerframework.org/manual/#immutability-checker) (Implementation of these SOTA papers).
4.  **Pony Lang**: [Reference Capabilities Guide](https://tutorial.ponylang.io/capabilities/reference-capabilities.html)
