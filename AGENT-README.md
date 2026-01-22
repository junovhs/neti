# SlopChop Autonomous Protocol

You are an autonomous engineer working in a **SlopChop** governed repository.
**Your Goal:** Complete tasks in `PLAN.md` using the SlopChop Transactional Workflow.

## 1. The Prime Directive

**NEVER commit to `main` directly.**
You must work inside the `slopchop-work` sandbox. You only earn the right to merge when `slopchop check` passes.

## 2. The Loop (Standard Operating Procedure)

Execute this exact loop for every task.

### PHASE 1: ISOLATE
1.  **Create and Read `PLAN.md`**. Pick the highest priority unchecked item.
2.  **Create your safe sandbox**:
    ```bash
    slopchop branch --force
    ```
    *(This creates or resets the `slopchop-work` branch. You are now safe to break things.)*

### PHASE 2: ACT
1.  **Modify the code directly.** Edit files, add features, refactor.
2.  **Use your tools**:
    *   `slopchop scan` -> Check token counts/complexity quickly.
    *   `slopchop map` -> See the repo structure.
    *   `slopchop map --deps` -> See the repo structure with dependancies.

### PHASE 3: VERIFY
1.  Run the gatekeeper:
    ```bash
    slopchop check
    ```
This will run the full test gamut, and is required to fully pass. 

2.  **IMMEDIATELY READ `slopchop-report.txt`**.
    *   **Ignore stdout.** It is truncated.
    *   The report contains the entirety of actual compiler errors, linter violations, and complexity scores. Terminal views get truncated - this does not. 

### PHASE 4: DECIDE
*   **IF FAILED (Red):**
    *   Analyze the violations in the report.
    *   **Refactor immediately.** (Extract functions, simplify logic).
    *   *Loop back to Phase 2 (Act).*
    *   *(Emergency: If you are totally stuck, run `slopchop abort` to delete the branch and start over WARNING: THIS IS MAXIMALLY DESTRUCTIVE, IT WILL LITERALLY ERASE ALL YOUR WORK, SO ENSURE WHATEVER WORK YOU WANTED TO SAVE HAS BEEN PUSHED. ASK THE USER BEFORE EVER RUNNING SLOPCHOP ABORT).*

*   **IF PASSED (Green):**
    1.  **Promote your work**:
        ```bash
        slopchop promote
        ```
        *(This automatically squashes and merges your work to main).*
    2.  **Update `PLAN.md`**: Mark the task as `[x]`.
    3.  **Push**:
        ```bash
        git push
        ```
    4.  *Loop back to Phase 1.*

---

## 3. The Laws of Physics

SlopChop enforces strict structural metrics. `slopchop check` will fail if you violate these.

| Metric | Limit | Fix |
| :--- | :--- | :--- |
| **File Tokens** | < 2,000 | **Split the file.** Create submodules. |
| **Cognitive Complexity** | ≤ 15 | **Extract methods.** Simplify branching. |
| **Nesting Depth** | ≤ 3 | **Use guard clauses** (return early). |
| **LCOM4** | = 1 | **Split the struct.** It lacks cohesion. |

**Context-Aware Profiling:**
If you see `systems` profile active (e.g. `unsafe`, `no_std`), limits relax, but **Safety Checks Escalate**. Every `unsafe` block *must* have a `// SAFETY:` comment.

## 4. Dishonorable Behavior

*   **Bypassing the Sandbox:** Never edit code on `main`. Always run `slopchop branch` first.
*   **Lazy Fixes:** Never add `#[allow(...)]` to silence SlopChop metrics. Refactor the code.
*   **Hallucinating Success:** Never run `slopchop promote` unless `slopchop check` was GREEN.
