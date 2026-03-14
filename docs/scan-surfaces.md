## Neti Scan Surface

  This is the breakdown of what Neti currently scans for.

  ### 1. Core Laws

  | Law | Condition Scanned | Description | Target Languages |
  |---|---|---|---|
  | LAW OF ATOMICITY | File size limit | Flags governed source files that exceed a configurable token budget using the cl100k_base tokenizer. Config, asset, and non-governed files are
  excluded. | All governed source files |
  | LAW OF COMPLEXITY | Cognitive complexity | Flags functions whose control flow and branching exceed the configured complexity budget. | Rust-first; non-Rust coverage depends on parser/
  query support |
  | LAW OF COMPLEXITY | Nesting depth | Flags functions with nesting deeper than the configured limit. | Rust-first; non-Rust coverage depends on parser/query support |
  | LAW OF COMPLEXITY | Function arity | Flags functions with too many parameters. | Rust-first; non-Rust coverage depends on parser/query support |
  | LAW OF COMPLEXITY | Function naming | Flags function names with too many words, using SWUM-aware identifier splitting. | Rust-first; non-Rust coverage depends on parser/query support |
  | LAW OF PARANOIA | Banned constructs | Flags .unwrap() and .expect() usage as disallowed error-handling shortcuts. | Rust |
  | LAW OF PARANOIA | Unsafe code constraints | Flags unsafe blocks without a preceding // SAFETY: justification, or flags all unsafe if configured to ban it. | Rust |
  | LAW OF INTEGRITY | Syntax errors | Flags malformed or unparseable code in supported parsers, with some guardrails for parser/version mismatches. | Any parsed language, tuned most heavily
  for Rust |

  ### 2. Anti-Patterns

  | Code | Category | Trigger | Description |
  |---|---|---|---|
  | P01 | Performance | Costly copy/allocation in loop | Flags materially costly copy/allocation patterns inside loops. |
  | P02 | Performance | String conversion in loop | Flags .to_string() / .to_owned() inside loop iterations. |
  | P03 | Performance | N+1 query | Flags likely DB calls inside loops when the loop variable is used in the call. |
  | P04 | Performance | Nested loops | Flags loop nesting that likely produces quadratic behavior. |
  | P06 | Performance | Linear search in loop | Flags lookup/search operations inside outer loops, indicating likely O(n*m) work. |
  | L02 | Logic | Inclusive length boundary | Flags <= / >= used against collection length for index bounds. |
  | L03 | Logic | Unchecked indexing | Flags indexing patterns that lack an evident bounds proof or guard. |
  | X01 | Security | SQL via string formatting | Flags SQL built with string formatting instead of parameterization. |
  | X02 | Security | Dynamic process execution | Flags risky dynamic process execution, especially shell-style invocation such as sh -c. |
  | X03 | Security | Hardcoded secret-like literal | Flags string literals assigned to bindings with names like key, secret, token, password, or auth. |
  | C03 | Concurrency | Lock across await | Flags lock guards that appear to remain live across an .await. |
  | C04 | Concurrency | Undocumented lock field | Flags lock-like synchronization fields lacking nearby explanatory documentation. |
  | R07 | Resource | Missing flush | Flags locally created BufWriter values that appear to go out of scope without flush(). |
  | S01 | State | Global mutable state | Flags static mut. |
  | S02 | State | Exported static state | Flags pub static shared state patterns. |
  | S03 | State | Global container singleton | Flags lazy_static-style global containers such as Mutex<Vec<_>> or Mutex<HashMap<_, _>>. |
  | I01 | Idiomatic | Manual From impl | Suggests a derive-based simplification for manual From impls when appropriate. |
  | I02 | Idiomatic | Duplicate match arms | Flags duplicate match arm bodies that could be fused when compatible. |
  | M03 | Semantic | Mutating getter | Flags get_ / is_ / has_ methods that take mutable self. |
  | M04 | Semantic | Non-bool predicate | Flags is_ / has_ / can_ / should_ methods that do not return bool. |
  | M05 | Semantic | Mutating calculator | Flags calculate_ / compute_ / count_ / sum_ methods that take mutable self. |

  ### 3. Deep Structural Metrics

  These are configurable governance signals, not universal defect claims. They are currently strongest on Rust and run in the deep-analysis path.

  | Metric | Name | What it scans for | Threshold style |
  |---|---|---|---|
  | LCOM4 | Lack of Cohesion of Methods | Types whose methods do not meaningfully share fields or internal behavior. | Fires when exceeding max_lcom4 |
  | CBO | Coupling Between Objects | Types with too many external dependencies. | Fires when exceeding max_cbo |
  | SFOUT | Structural Fan-Out | Types or methods with excessive outgoing call fan-out. | Fires when exceeding max_sfout |
  | AHF | Attribute Hiding Factor | Types that expose too much state publicly rather than encapsulating it. | Fires when falling below min_ahf |

  ### 4. Locality / Architecture Graph Analysis

  This is separate from the AST anti-patterns. It analyzes dependency shape and module topology. Coverage varies by language ecosystem and import-resolution support.

  | Finding | What it scans for |
  |---|---|
  | Dependency cycles | Circular dependency paths in the locality graph. Depending on configuration, these may warn or block. |
  | ENCAPSULATION_BREACH | Importing internal implementation files instead of crossing a public module/API boundary. |
  | GOD_MODULE | Files with too many cross-boundary dependencies. |
  | MISSING_HUB | High-fan-in files that should likely be treated as explicit hubs. |
  | SIDEWAYS_DEP | Cross-module dependencies that jump laterally without acceptable routing. |
  | UPWARD_DEP | Dependencies that violate inferred architectural layering. |

  ### Language Positioning

  - Neti is strongest on Rust today.
  - Non-Rust support exists, but it is narrower and less uniform.
  - The clearest shared cross-language detections today are P06 and L02.
  - LAW OF ATOMICITY applies broadly to governed source files regardless of language.
  - Structural metrics and many semantic/concurrency/security rules are primarily Rust-centric.
