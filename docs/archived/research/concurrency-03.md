# SOTA Report: Safe vs Dangerous Async Shared State (Concurrency-03)

**Date**: January 2026  
**Subject**: Patterns and anti-patterns for shared state in asynchronous programming, and their static analysis indicators.

## 1. Executive Summary
The primary danger in asynchronous programming is the **Execution-Thread Mismatch**. State-of-the-Art (SOTA) analysis recognizes that "Thread Safety" does not equal "Async Safety." This report categorizes shared state patterns by their risk profile and details the static analysis signals used to distinguish them.

---

## 2. Dangerous Patterns: Implicit Context Drift

### 2.1 Thread-Local Storage (TLS) Escape
Traditional TLS (`ThreadLocal<T>`) is dangerous in async contexts because a task may resume on a different thread after an `await`.

*   **SOTA Pitfall**: **Context Loss**. If data is stored in TLS at line \(N\) and accessed at line \(N+2\) (after an `await`), the data may be null or, worse, contain data from an unrelated task currently inhabiting that thread.
*   **Static Signal**: Detection of `await` points within the scope of a TLS variable access.
*   **SOTA Fix**: Migration to **Async-Local Storage** (`AsyncLocal<T>` in .NET, `ContextVars` in Python), which flows with the logical execution context rather than the physical thread.

### 2.2 Unprotected Concurrent Modification
Iterating over a collection while performing asynchronous operations that might modify said collection.

*   **Pattern**: `for (var item in list) { await doWork(); list.Add(newItem); }`
*   **Static Detection**: **CCFG (Concurrency Control Flow Graph)** analysis. The tool identifies that the loop's "Next" operation is separated from its "Write" operation by a suspension point, creating an interleaving gap where the iterator's internal state becomes invalid.

---

## 3. Safe Patterns: Intentional Isolation

### 3.1 The Actor Model (Isolation by Design)
The most robust SOTA pattern for async state.

*   **Mechanism**: **Private State + Async Message Passing**.
    1.  No two units of code share memory.
    2.  State changes are handled sequentially within a single actor.
*   **SOTA Insight**: Software analysis of Actor systems focuses on **Protocol Correctness** (message ordering) rather than low-level race conditions.
*   **Citation**: Hewitt, C. (1973). *A Universal Modular Actor Formalism for Artificial Intelligence*. (Modern SOTA: Akka, Erlang/Elixir).

### 3.2 Immutable Snapshots & Value Types
Pass-by-value architectures (common in Swift and modern C++).

*   **Mechanism**: When an async task starts, it receives a **Deep Copy** or an **Immutable Reference** to the state.
*   **Static Signal**: Enforcement of `val` or `readonly` capabilities. The analyzer proves that the async closure captures only immutable or isolated data, making races mathematically impossible.

---

## 4. Analysis Contrast: SOTA Techniques

| Category | Dangerous Signal | Safe Signal |
| :--- | :--- | :--- |
| **Storage** | `ThreadLocal` + `await` | `AsyncLocal` / Parameter Passing |
| **Access** | Shared Pointer + `await` | Deep Copy / Ownership Transfer |
| **Logic** | Manual Mutex across `await` | SemaphoreSlim / tokio::sync |
| **Architecture** | Global Singleton | Actor / Service Registry |

---

## 5. Modern Research Frontier: Dynamic Region Ownership
Recent SOTA research in languages like Python and JS involves **Dynamic Region Ownership**.

*   **Mechanism**: The runtime assigns a "Region ID" to chunks of memory. A task can only modify a region if it "owns" it. If a task `awaits`, it might lose or explicitly yield ownership.
*   **Impact**: Detects "Async Race Conditions" at the moment they occur with deterministic failures, rather than silent data corruption.
*   **Citation**: (2024). *Dynamic Region Ownership for Concurrency Safety*.

---

## 6. References
1.  **AsyncLocal Pitfalls**: [Microsoft Documentation on Context Flow](https://learn.microsoft.com/en-us/dotnet/api/system.threading.asynclocal-1)
2.  **Actor Model**: [The Actor Model of Computation](https://arxiv.org/abs/1008.1459)
3.  **Concurrency Safety**: [Dynamic Region Ownership Research](https://github.com/microsoft/region-ownership)
4.  **Async-TAJS**: [Static Analysis for Asynchronous JS](http://www.brics.dk/TAJS/)
