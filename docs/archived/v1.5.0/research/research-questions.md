## State

1. What static analysis techniques detect deep mutability (object property mutation) without full data flow analysis?
2. What metrics exist for measuring "state ownership spread" across a codebase, beyond basic coupling scores?
3. Are there AST patterns that predict state bugs before they manifest — leading indicators rather than violations?
4. What tools automatically refactor global state to dependency injection, and how do they decide injection points?

## Concurrency

1. What static patterns detect race conditions in single-threaded event loop languages like JavaScript?
2. What techniques detect resources held across await points without runtime instrumentation?
3. Are there heuristics for distinguishing "safe" shared state from "dangerous" shared state in async code?
4. What modern tools detect deadlock potential from code structure alone, not runtime traces?

## Resource

1. What patterns detect event listener leaks or subscription leaks in browser/Node.js code statically?
2. How can static analysis detect unbounded collection growth (push without clear) as a memory leak risk?
3. What AST patterns indicate "resource acquired too early" or "held too long" anti-patterns?
4. Are there lightweight alternatives to escape analysis for detecting leaked references?

## Security

1. What modern taint analysis approaches work on dynamic languages without type annotations?
2. What regex or entropy-based techniques detect hardcoded secrets with lowest false positive rates?
3. What patterns detect unsafe deserialization or prototype pollution vulnerabilities in JavaScript?
4. Are there static techniques to flag "user input reaches dangerous sink" without full taint tracking?

## Performance

1. Can AST patterns detect algorithmic complexity (O(n²) nested loops) without symbolic execution?
2. What heuristics identify "hot path" code that deserves extra scrutiny for allocations?
3. What patterns detect N+1 query problems or repeated identical async calls statically?
4. Are there lightweight proxies for "this code will be slow" that don't require benchmarking?

## Semantic

1. What NLP or embedding techniques detect function name vs function body misalignment?
2. Are there metrics for "function does too many things" beyond cyclomatic complexity?
3. What patterns detect "dead parameters" — arguments that are passed but never used meaningfully?
4. Can static analysis detect when a function's return value semantically contradicts its name?