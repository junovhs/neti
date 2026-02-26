//! Static educational guidance per rule code.

/// Static educational guidance per rule.
pub(crate) struct RuleGuidance {
    pub(crate) why: &'static str,
    pub(crate) fix: &'static str,
}

/// Returns educational guidance for a rule code, if available.
pub(crate) fn get_guidance(rule: &str) -> Option<RuleGuidance> {
    Some(match rule {
        "P01" => RuleGuidance {
            why: "Cloning/copying inside a loop allocates on every iteration, scaling linearly with iteration count.",
            fix: "Hoist the allocation before the loop, use a reference or borrow, or confirm the copy is cheap (primitives, small structs, reference-counted pointers).",
        },
        "P02" => RuleGuidance {
            why: "String conversion inside a loop allocates a new String on every iteration.",
            fix: "Hoist the conversion before the loop, or operate on borrowed string slices (&str).",
        },
        "P04" => RuleGuidance {
            why: "Nested loops produce O(n²) complexity, which scales poorly with input size.",
            fix: "Replace the inner loop with a lookup structure (HashMap/HashSet) for O(n) total, or confirm the inner loop is bounded to a small constant.",
        },
        "P06" => RuleGuidance {
            why: "Linear search (.find/.position/.index) inside a loop produces O(n·m) complexity.",
            fix: "Pre-build a lookup structure (HashSet/HashMap/dict/Set) for O(1) access, or confirm the inner collection is bounded to a small constant size.",
        },
        "L02" => RuleGuidance {
            why: "Using <= or >= with .len() in index bounds can reach len, which is one past the last valid index.",
            fix: "Use < len for upper bounds on indices. The valid index range is 0..len-1.",
        },
        "L03" => RuleGuidance {
            why: "Indexing without a bounds proof panics on empty or undersized collections at runtime.",
            fix: "Use safe accessors (.first()/.get()), add an emptiness guard, or prove the collection size is guaranteed by construction (fixed-size array, chunks_exact).",
        },
        "X01" => RuleGuidance {
            why: "Building SQL from string formatting allows injection when inputs are user-controlled.",
            fix: "Use parameterized queries (? placeholders) with your database driver's bind API.",
        },
        "X02" => RuleGuidance {
            why: "Executing external commands with dynamic arguments risks injection (shell) or untrusted binary resolution (direct exec).",
            fix: "For shell commands: validate and sanitize inputs, or avoid shell invocation entirely. For direct exec: use absolute paths, allowlists, or signature verification.",
        },
        "C03" => RuleGuidance {
            why: "Holding a lock guard across an await point blocks the executor thread (sync mutex) or starves other tasks (async mutex).",
            fix: "Scope the guard so it drops before the await, or extract the critical section into a synchronous helper function.",
        },
        "C04" => RuleGuidance {
            why: "Synchronization primitives without documentation make concurrent code harder to reason about and audit.",
            fix: "Add a comment explaining what the lock protects and the expected contention pattern.",
        },
        "I01" => RuleGuidance {
            why: "Manual From implementations are boilerplate that can be generated with derive macros.",
            fix: "Use derive_more::From if your project already depends on proc macros. Manual impls are perfectly fine for zero-dependency crates.",
        },
        "I02" => RuleGuidance {
            why: "Duplicate match arm bodies indicate arms that could be combined with the | pattern.",
            fix: "Combine arms: `A | B => shared_body`. Only valid when bindings have compatible types.",
        },
        "M03" | "M04" | "M05" => RuleGuidance {
            why: "Function name implies a contract (getter, predicate, pure computation) that the implementation violates.",
            fix: "Rename the function to match its behavior, or refactor the implementation to match its name.",
        },
        "R07" => RuleGuidance {
            why: "Buffered writers that are dropped without flushing may silently lose data.",
            fix: "Call .flush() explicitly before the writer goes out of scope, or return it so the caller controls lifetime.",
        },
        "S01" | "S02" | "S03" => RuleGuidance {
            why: "Global mutable state creates hidden coupling and makes code harder to test and reason about.",
            fix: "Pass state explicitly via function parameters, or use dependency injection patterns.",
        },
        "LAW OF PARANOIA" => RuleGuidance {
            why: "Unsafe blocks must document their safety invariants so reviewers can verify correctness.",
            fix: "Add a // SAFETY: comment immediately above the unsafe block explaining why the invariants hold.",
        },
        "LAW OF ATOMICITY" => RuleGuidance {
            why: "Files beyond the token limit are too large for a single unit of work, increasing cognitive load and merge conflict risk.",
            fix: "Split the file into smaller, focused modules. Extract related functions into their own files.",
        },
        "LAW OF INTEGRITY" => RuleGuidance {
            why: "Syntax errors prevent analysis and indicate malformed or unparseable code.",
            fix: "Fix the syntax error, or if this is valid modern syntax that Neti's parser doesn't support, file an issue.",
        },
        _ => return None,
    })
}
