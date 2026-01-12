# SOTA Report: Deadlock Detection from Structure (Concurrency-04)

**Date**: January 2026  
**Subject**: Formal and structural models for identifying deadlocks in complex asynchronous and distributed systems.

## 1. Executive Summary
Deadlock detection has evolved from simple cycle detection in local **Wait-For Graphs (WFG)** to sophisticated **Petri Net** structural analysis and **Formal Model Checking** with TLA+. In asynchronous systems, the "Waiting" relationship is often implicit in task continuations, necessitating the use of **Continuation Scheduling Graphs** to surface hidden circular dependencies.

---

## 2. Evolution of the Wait-For Graph (WFG)

### 2.1 The Classic WFG
*   **Mechanism**: A directed graph where nodes are processes and edges indicate that Process A is waiting for a resource held by Process B.
*   **SOTA Logic**: A cycle in the WFG is a necessary and sufficient condition for a deadlock in a single-unit resource system.

### 2.2 The Continuation Scheduling Graph (CSG)
Asynchronous code (e.g., C# Task Parallel Library) does not block threads in the traditional sense, but it does block "Progress."

*   **SOTA Mechanism**: The CSG tracks **Logical Tasks** and their **Scheduling Contexts**.
*   **Deadlock Pattern**: Task A awaits Task B, but Task B is queued for a `SynchronizationContext` (e.g., UI Thread) that is currently busy executing the synchronous wait for Task A.
*   **Detection**: Identification of cyclic dependencies between logical tasks and the physical threads required for their continuations.
*   **Citation**: (2023). *DeadWait: Static Deadlock Detection for Asynchronous C# Programs*.

---

## 3. Specialized Formal Models

### 3.1 Petri Nets: Siphons and Traps
Petri Nets provide a mathematical basis for analyzing resource flow.

*   **Siphon Detection**: A "Siphon" is a set of places in a Petri Net that, if emptied of tokens, can never be replenished.
*   **SOTA Rule**: Every deadlock in a resource-allocation system corresponds to an "Empty Siphon" in its Petri Net representation.
*   **Fix Strategy**: Adding "Supervisor Transitions" that prevent siphons from becoming empty, effectively preventing deadlocks by structure rather than runtime check.
*   **Citation**: (2021). *Deadlock Analysis of Petri Nets using Siphons and Traps*.

### 3.2 TLA+ (Temporal Logic of Actions)
Used by Amazon (AWS) and Microsoft for verifying complex async protocols.

*   **Mechanism**: Model Checking. TLA+ exhaustively explores every possible state of the system's logic (including non-deterministic event orderings).
*   **Detection**: TLA+ identifies states where "Next" is null for all processes, indicating a global deadlock.
*   **Citation**: Lamport, L. (2002). *Specifying Systems: The TLA+ Language and Tools*.

---

## 4. Structural Avoidance Algorithms
When a deadlock can be detected from the structure of a request, these algorithms prevent the cycle.

*   **Wait-Die (Non-Preemptive)**: An older process waits for a younger one; a younger one "dies" (aborts) if it requests a resource from an older one.
*   **Wound-Wait (Preemptive)**: An older process "wounds" (aborts) a younger one to take its resource; a younger one waits for an older one.
*   **SOTA Choice**: Wound-Wait is generally preferred in high-contention async systems as it results in fewer rollbacks and guarantees progress for the "oldest" task (avoiding starvation).

---

## 5. Synthesis: Detection vs. Prevention

| Technique | Level | Goal |
| :--- | :--- | :--- |
| **Cycle Check (WFG)** | Local / Runtime | Detection |
| **Siphon Analysis** | Structural / Static | Prevention |
| **Model Checking (TLA+)** | Logic / Formal | Verification |
| **Wound-Wait** | Algorithmic | Avoidance |

---

## 6. References
1.  **DeadWait**: [ASE 2023 Research](https://ieeexplore.ieee.org/document/10298357)
2.  **Petri Nets for FMS**: [ResearchGate: Deadlock Prevention](https://www.researchgate.net/publication/224214227_Petri_net-based_deadlock_prevention_policies_for_flexible_manufacturing_systems)
3.  **Specifying Systems**: [Leslie Lamport's TLA+ Site](https://lamport.azurewebsites.net/tla/tla.html)
4.  **Wait-Die and Wound-Wait**: [Database System Concepts (SOTA Algorithms)](https://www.db-book.com/)
