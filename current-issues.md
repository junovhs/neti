[❯ slopchop scan
error: Boundary uses `<=`/`>=` with `.len()`
  --> src/graph/rank/graph.rs:64
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: N+1 query: `get` in loop
  --> src/graph/rank/pagerank.rs:60
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `file` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `get` in loop
  --> src/graph/rank/pagerank.rs:87
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `source` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: Calculator `compute_entropy` takes &mut self
  --> src/graph/locality/validator.rs:52
   |
   = M05: Action required
   |
   = ANALYSIS:
   |   Pure calculations should not mutate.
   |
   = SUGGESTION: Remove mutation or rename.

error: N+1 query: `get` in loop
  --> src/graph/locality/report.rs:62
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `kind` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `get` in loop
  --> src/graph/locality/layers.rs:76
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `node` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/graph/locality/analysis/metrics.rs:36
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/graph/locality/analysis/metrics.rs:62
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: N+1 query: `to)` in loop
  --> src/graph/locality/analysis/mod.rs:38
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `edge` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/audit/similarity.rs:50
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/audit/similarity.rs:50
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/audit/similarity.rs:50
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/audit/similarity.rs:63
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/audit/display.rs:27
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: N+1 query: `find` in loop
  --> src/audit/similarity_core.rs:65
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `i` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/audit/similarity_core.rs:42
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/audit/similarity_core.rs:69
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/audit/parameterize.rs:51
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/apply/validator.rs:126
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/apply/validator.rs:126
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/apply/validator.rs:126
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/apply/messages.rs:138
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/apply/process_runner.rs:79
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: N+1 query: `get` in loop
  --> src/apply/patch/diagnostics.rs:53
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `i` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `get` in loop
  --> src/apply/patch/diagnostics.rs:54
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `i` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/apply/patch/diagnostics.rs:148
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/apply/patch/diagnostics.rs:156
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/analysis/checks/naming.rs:52
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: N+1 query: `path)` in loop
  --> src/analysis/logic.rs:33
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `r` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `first` in loop
  --> src/analysis/v2/rust.rs:93
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `index == 0)` in loop
  --> src/analysis/v2/patterns/performance.rs:39
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/performance.rs:43
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/performance.rs:61
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/performance.rs:62
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/performance.rs:115
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/performance.rs:116
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `first` in loop
  --> src/analysis/v2/patterns/performance.rs:163
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `index == 1)` in loop
  --> src/analysis/v2/patterns/state.rs:26
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `index == 1)` in loop
  --> src/analysis/v2/patterns/state.rs:114
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/analysis/v2/patterns/state.rs:159
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: N+1 query: `first` in loop
  --> src/analysis/v2/patterns/logic.rs:22
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `first` in loop
  --> src/analysis/v2/patterns/logic.rs:55
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `first` in loop
  --> src/analysis/v2/patterns/logic.rs:82
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `first` in loop
  --> src/analysis/v2/patterns/idiomatic.rs:26
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/idiomatic.rs:55
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/idiomatic.rs:56
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/analysis/v2/patterns/idiomatic.rs:93
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/semantic.rs:31
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/semantic.rs:32
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/semantic.rs:33
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/semantic.rs:65
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/semantic.rs:66
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/semantic.rs:67
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/semantic.rs:99
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/semantic.rs:100
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/semantic.rs:101
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/resource.rs:28
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `index == 0)` in loop
  --> src/analysis/v2/patterns/db_patterns.rs:29
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/db_patterns.rs:33
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/db_patterns.rs:66
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `index == 1)` in loop
  --> src/analysis/v2/patterns/security.rs:26
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/security.rs:62
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `find` in loop
  --> src/analysis/v2/patterns/security.rs:63
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `index == 1)` in loop
  --> src/analysis/v2/patterns/security.rs:155
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `m` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `get` in loop
  --> src/analysis/safety.rs:99
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `i` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

X SlopChop found 65 violations in 9.393s.

❯ cargo clippy --all-targets -- -D warnings -W clippy::pedantic -W clippy::unwrap_used -W clippy::expect_used -W clippy::indexing_slicing -A clippy::struct_excessive_bools -A clippy::module_name_repetitions -A clippy::missing_errors_doc -A clippy::must_use_candidate
    Checking slopchop v1.6.0 (/home/juno/slopchop)
error: indexing may panic
   --> src/apply/patch.rs:123:13
    |
123 |             matches[0].0,
    |             ^^^^^^^^^^
    |
    = help: consider using `.get(n)` or `.get_mut(n)` instead
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing
    = note: `-D clippy::indexing-slicing` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::indexing_slicing)]`

error: indexing may panic
   --> src/apply/patch.rs:160:17
    |
160 |                 matches[0].0,
    |                 ^^^^^^^^^^
    |
    = help: consider using `.get(n)` or `.get_mut(n)` instead
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: indexing may panic
  --> src/apply/parser.rs:89:27
   |
89 |         assert!(matches!(&blocks[0], Block::Plan(c) if c == "My Plan"));
   |                           ^^^^^^^^^
   |
   = help: consider using `.get(n)` or `.get_mut(n)` instead
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing
   = note: `-D clippy::indexing-slicing` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(clippy::indexing_slicing)]`

error: indexing may panic
  --> src/apply/parser.rs:90:27
   |
90 |         assert!(matches!(&blocks[1], Block::Manifest(c) if c == "file.rs"));
   |                           ^^^^^^^^^
   |
   = help: consider using `.get(n)` or `.get_mut(n)` instead
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: indexing may panic
   --> src/apply/parser.rs:102:16
    |
102 |         match &blocks[0] {
    |                ^^^^^^^^^
    |
    = help: consider using `.get(n)` or `.get_mut(n)` instead
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: indexing may panic
   --> src/apply/parser.rs:109:16
    |
109 |         match &blocks[1] {
    |                ^^^^^^^^^
    |
    = help: consider using `.get(n)` or `.get_mut(n)` instead
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: used `unwrap_err()` on a `Result` value
   --> src/apply/parser.rs:122:19
    |
122 |         let err = parse(&input).unwrap_err();
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: if this value is an `Ok`, it will panic
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#unwrap_used
    = note: `-D clippy::unwrap-used` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::unwrap_used)]`

error: indexing may panic
   --> src/apply/parser.rs:135:27
    |
135 |         assert!(matches!(&blocks[0], Block::Plan(c) if c == "Plan"));
    |                           ^^^^^^^^^
    |
    = help: consider using `.get(n)` or `.get_mut(n)` instead
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: indexing may panic
   --> src/apply/parser.rs:136:27
    |
136 |         assert!(matches!(&blocks[1], Block::Manifest(c) if c == "Man"));
    |                           ^^^^^^^^^
    |
    = help: consider using `.get(n)` or `.get_mut(n)` instead
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: indexing may panic
   --> src/apply/parser.rs:137:16
    |
137 |         match &blocks[2] {
    |                ^^^^^^^^^
    |
    = help: consider using `.get(n)` or `.get_mut(n)` instead
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: indexing may panic
   --> src/apply/parser.rs:152:16
    |
152 |         match &blocks[0] {
    |                ^^^^^^^^^
    |
    = help: consider using `.get(n)` or `.get_mut(n)` instead
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: indexing may panic
  --> src/audit/display.rs:28:26
   |
28 |             let val_a = &hole.variants[0];
   |                          ^^^^^^^^^^^^^^^^
   |
   = help: consider using `.get(n)` or `.get_mut(n)` instead
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: indexing may panic
  --> src/audit/display.rs:29:26
   |
29 |             let val_b = &hole.variants[1];
   |                          ^^^^^^^^^^^^^^^^
   |
   = help: consider using `.get(n)` or `.get_mut(n)` instead
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: indexing may panic
  --> src/apply/patch/tests.rs:11:14
   |
11 |     let i = &instrs[0];
   |              ^^^^^^^^^
   |
   = help: consider using `.get(n)` or `.get_mut(n)` instead
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: used `unwrap_err()` on a `Result` value
  --> src/apply/patch/tests.rs:36:15
   |
36 |     let err = apply(original, patch).unwrap_err();
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: if this value is an `Ok`, it will panic
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#unwrap_used

error: indexing may panic
   --> src/apply/patch.rs:123:13
    |
123 |             matches[0].0,
    |             ^^^^^^^^^^
    |
    = help: consider using `.get(n)` or `.get_mut(n)` instead
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: indexing may panic
  --> src/audit/similarity.rs:89:13
   |
89 |             files[0]
   |             ^^^^^^^^
   |
   = help: consider using `.get(n)` or `.get_mut(n)` instead
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: indexing may panic
  --> src/cli/config_ui/items.rs:49:9
   |
49 |         LABELS[self as usize]
   |         ^^^^^^^^^^^^^^^^^^^^^
   |
   = help: consider using `.get(n)` or `.get_mut(n)` instead
   = note: the suggestion might not be applicable in constant blocks
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: indexing may panic
  --> src/cli/config_ui/logic.rs:81:16
   |
81 |     let item = editor.items()[selected];
   |                ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: consider using `.get(n)` or `.get_mut(n)` instead
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: slicing may panic
   --> src/graph/locality/cycles.rs:102:25
    |
102 |         let mut cycle = state.path_stack[pos..].to_vec();
    |                         ^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: consider using `.get(n..)` or .get_mut(n..)` instead
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: indexing may panic
  --> src/graph/locality/distance.rs:24:12
   |
24 |         if a[i] == b[i] {
   |            ^^^^
   |
   = help: consider using `.get(n)` or `.get_mut(n)` instead
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: indexing may panic
  --> src/graph/locality/distance.rs:24:20
   |
24 |         if a[i] == b[i] {
   |                    ^^^^
   |
   = help: consider using `.get(n)` or `.get_mut(n)` instead
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: slicing may panic
   --> src/graph/locality/report.rs:185:43
    |
185 |             format!("{}, ... ({} total)", mods[0..8].join(", "), mods.len())
    |                                           ^^^^^^^^^^
    |
    = help: consider using `.get(n..m)` or `.get_mut(n..m)` instead
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: indexing may panic
   --> src/graph/locality/cycles.rs:132:20
    |
132 |         assert_eq!(cycles[0].len(), 3); // a -> b -> a
    |                    ^^^^^^^^^
    |
    = help: consider using `.get(n)` or `.get_mut(n)` instead
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: indexing may panic
   --> src/signatures/mod.rs:169:23
    |
169 |     let mut current = ranges[0].clone();
    |                       ^^^^^^^^^
    |
    = help: consider using `.get(n)` or `.get_mut(n)` instead
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: indexing may panic
  --> src/skeleton.rs:66:24
   |
66 |         let current = &ranges[i];
   |                        ^^^^^^^^^
   |
   = help: consider using `.get(n)` or `.get_mut(n)` instead
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#indexing_slicing

error: could not compile `slopchop` (lib) due to 13 previous errors
warning: build failed, waiting for other jobs to finish...
error: could not compile `slopchop` (lib test) due to 25 previous errors
❯ cargo test
   Compiling slopchop v1.6.0 (/home/juno/slopchop)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.37s
     Running unittests src/lib.rs (target/debug/deps/slopchop_core-632aff0e3b230665)

running 79 tests
test analysis::checks::syntax::tests::test_valid_rust ... ok
test analysis::v2::cognitive::tests::test_boolean_ops ... ok
test analysis::v2::cognitive::tests::test_else_if_flattening ... ok
test analysis::checks::syntax::tests::test_rust_error ... ok
test analysis::v2::cognitive::tests::test_if_statement ... ok
test analysis::v2::cognitive::tests::test_linear_flow ... ok
test analysis::v2::cognitive::tests::test_nested_if ... ok
test apply::blocks::tests::test_clean_empty_prefix ... ok
test apply::blocks::tests::test_create_file_block ... ok
test apply::blocks::tests::test_clean_line_fallback ... ok
test apply::blocks::tests::test_create_plan_block ... ok
test apply::blocks::tests::test_clean_with_prefix ... ok
test apply::blocks::tests::test_rejects_keyword_paths ... ok
test apply::messages::tests::test_feedback_truncation_utf8 ... ok
test apply::messages::tests::test_floor_char_boundary ... ok
test analysis::v2::patterns::resource::tests::r07_flag_missing_flush ... ok
test analysis::v2::patterns::resource::tests::r07_skip_returned_writer ... ok
test apply::patch::common::hash_tests::test_eol_normalization ... ok
test apply::parser::tests::test_inconsistent_prefix_parsing ... ok
test apply::patch::tests::test_diagnostic_ambiguous ... ok
test apply::patch::tests::test_diagnostic_zero_match_probe ... ok
test apply::patch::tests::test_v1_apply ... ok
test apply::parser::tests::test_parse_plan_and_manifest ... ok
test audit::fp_similarity::tests::test_cfg_hash_equivalence ... ok
test apply::patch::tests::test_v1_parse ... ok
test audit::fp_similarity::tests::test_different_cfg_similar_metrics ... ok
test branch::tests::test_work_branch_name ... ok
test audit::fp_similarity::tests::test_exact_match ... ok
test analysis::v2::patterns::resource::tests::r07_skip_with_flush ... ok
test apply::parser::tests::test_parse_file_and_patch ... ok
test analysis::v2::patterns::idiomatic::tests::i02_flag_duplicate_arms ... ok
test apply::parser::tests::test_rejects_keyword_path ... ok
test apply::patch::common::hash_tests::test_hash_stability ... ok
test graph::locality::classifier::tests::test_deadwood ... ok
test graph::locality::classifier::tests::test_god_module ... ok
test graph::locality::classifier::tests::test_stable_hub ... ok
test graph::locality::classifier::tests::test_volatile_leaf ... ok
test graph::locality::coupling::tests::test_compute_coupling ... ok
test graph::locality::cycles::tests::test_diamond_dag_no_cycle ... ok
test graph::locality::cycles::tests::test_simple_cycle ... ok
test graph::locality::cycles::tests::test_no_cycles ... ok
test graph::locality::distance::tests::test_deep_hierarchy ... ok
test graph::locality::exemptions::tests::test_crate_root ... ok
test graph::locality::distance::tests::test_same_directory ... ok
test graph::locality::exemptions::tests::test_parent_reexport ... ok
test graph::locality::distance::tests::test_sibling_directories ... ok
test graph::locality::exemptions::tests::test_shared_infrastructure ... ok
test graph::tsconfig::tests::test_match_pattern ... ok
test graph::locality::exemptions::tests::test_vertical_routing ... ok
test graph::tsconfig::tests::test_strip_comments ... ok
test lang::tests::test_from_ext ... ok
test map::tests::test_format_size ... ok
test map::tests::test_format_tokens ... ok
test analysis::v2::patterns::idiomatic::tests::i01_skip_error_from ... ok
test mutate::mutations::tests::test_boolean_mutations ... ok
test mutate::discovery::tests::test_discover_finds_operators ... ok
test mutate::mutations::tests::test_logical_mutations ... ok
test mutate::mutations::tests::test_no_mutation ... ok
test analysis::v2::patterns::idiomatic::tests::i02_skip_unique_arms ... ok
test analysis::v2::patterns::idiomatic::tests::i01_flag_simple_from ... ok
test mutate::mutations::tests::test_comparison_mutations ... ok
test graph::defs::extract::tests::test_python_defs ... ok
test analysis::v2::patterns::logic::tests::l03_flag_first_unwrap ... ok
test analysis::v2::patterns::logic::tests::l03_skip_with_empty_check ... ok
test analysis::v2::patterns::logic::tests::l03_flag_index_zero ... ok
test analysis::v2::patterns::logic::tests::l02_flag_lte_len ... ok
test graph::defs::extract::tests::test_rust_defs ... ok
test apply::parser::tests::test_tolerant_parsing ... ok
test analysis::v2::patterns::semantic::tests::m03_flag_getter_with_mut ... ok
test analysis::v2::patterns::semantic::tests::m04_flag_is_returning_string ... ok
test analysis::v2::patterns::semantic::tests::m03_skip_getter_without_mut ... ok
test analysis::v2::patterns::semantic::tests::m05_flag_calculate_with_mut ... ok
test analysis::v2::patterns::semantic::tests::m04_skip_is_returning_bool ... ok
test analysis::v2::patterns::db_patterns::tests::p03_flag_fetch_in_loop ... ok
test analysis::v2::patterns::db_patterns::tests::p03_flag_query_in_loop ... ok
test analysis::v2::patterns::db_patterns::tests::p03_skip_no_loop_var ... ok
test analysis::v2::patterns::db_patterns::tests::p03_skip_unrelated_call ... ok
test graph::imports::tests::test_extract_imports ... ok
test map::tests::test_build_tree_structure ... ok

test result: ok. 79 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s

     Running unittests src/bin/slopchop.rs (target/debug/deps/slopchop-8373475cf121641c)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests slopchop_core

running 1 test
test src/mutate/mod.rs - mutate (line 17) ... ignored

test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
](error: Boundary uses `<=`/`>=` with `.len()`
  --> src/graph/rank/graph.rs:64
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: N+1 query: `get` in loop
  --> src/graph/rank/pagerank.rs:60
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `file` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `get` in loop
  --> src/graph/rank/pagerank.rs:87
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `source` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: Calculator `compute_entropy` takes &mut self
  --> src/graph/locality/validator.rs:52
   |
   = M05: Action required
   |
   = ANALYSIS:
   |   Pure calculations should not mutate.
   |
   = SUGGESTION: Remove mutation or rename.

error: N+1 query: `get` in loop
  --> src/graph/locality/report.rs:62
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `kind` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `get` in loop
  --> src/graph/locality/layers.rs:76
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `node` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/graph/locality/analysis/metrics.rs:36
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/graph/locality/analysis/metrics.rs:62
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: N+1 query: `to)` in loop
  --> src/graph/locality/analysis/mod.rs:38
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `edge` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/audit/similarity.rs:50
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/audit/similarity.rs:50
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/audit/similarity.rs:50
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/audit/similarity.rs:63
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/audit/display.rs:28
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: N+1 query: `find` in loop
  --> src/audit/similarity_core.rs:65
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `i` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/audit/similarity_core.rs:42
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/audit/similarity_core.rs:69
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/audit/parameterize.rs:51
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/apply/validator.rs:126
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/apply/validator.rs:126
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/apply/validator.rs:126
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/apply/messages.rs:138
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/apply/process_runner.rs:79
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: N+1 query: `get` in loop
  --> src/apply/patch/diagnostics.rs:53
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `i` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: N+1 query: `get` in loop
  --> src/apply/patch/diagnostics.rs:54
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `i` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/apply/patch/diagnostics.rs:148
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/apply/patch/diagnostics.rs:156
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/analysis/checks/naming.rs:52
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: N+1 query: `path)` in loop
  --> src/analysis/logic.rs:33
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `r` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/analysis/v2/patterns/state.rs:159
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Boundary uses `<=`/`>=` with `.len()`
  --> src/analysis/v2/patterns/idiomatic.rs:93
   |
   = L02: Action required
   |
   = ANALYSIS:
   |   May cause off-by-one. Indices are 0..len-1.
   |
   = SUGGESTION: Usually want `< len` not `<= len`.

error: Function 'is_iterator_pattern' has cognitive complexity 18 (Max: 15)
  --> src/analysis/v2/patterns/db_patterns.rs:100
   |
   = LAW OF COMPLEXITY: Action required
   |
   = ANALYSIS:
   |   Cognitive score: 18
   |
   = SUGGESTION: Break logic into smaller, linear functions.

error: N+1 query: `get` in loop
  --> src/analysis/safety.rs:99
   |
   = P03: Action required
   |
   = ANALYSIS:
   |   DB call inside loop causes N+1 queries.
   |   Loop variable `i` used in call.
   |
   = SUGGESTION: Batch the query or use JOIN/IN.

error: File size is 10023 tokens (Limit: 2000)
  --> current-issues.md:1
   |
   = LAW OF ATOMICITY: Action required

X SlopChop found 34 violations in 9.378s.)
