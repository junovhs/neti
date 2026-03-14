# Law of Locality

## Definition

The Law of Locality is Neti's architectural rule that code should depend mostly on code that is nearby, structurally appropriate, and directionally sane.

It is not a single AST lint.

It is a derived graph-level metric and judgment system built from multiple lower-level signals:

- dependency edges between files
- topological distance between files
- fan-in and fan-out coupling
- node role classification
- cycle detection
- inferred architectural layers

The goal is to distinguish normal architecture from spaghetti dependencies.

## Core Idea

A codebase has good locality when:

- dependencies stay physically and conceptually near each other
- shared dependencies are promoted into stable hubs instead of being imported sideways everywhere
- imports respect architectural direction
- internal implementation files are not reached through back doors
- the dependency graph stays acyclic

The Law of Locality treats architecture as a graph problem, not just a file-by-file style problem.

## What Neti Calculates

Neti builds a dependency graph from source files and computes several values for each edge and node.

### 1. Dependency Distance

For a dependency edge `A -> B`, Neti computes a directory-tree distance using the lowest common ancestor of the two paths.

Implemented in [src/graph/locality/distance.rs](/home/juno/neti/src/graph/locality/distance.rs).

Formula:

`D(a, b) = (depth(a) - depth(LCA)) + (depth(b) - depth(LCA))`

Interpretation:

- low distance means the dependency is nearby
- high distance means the dependency jumps across the tree

### 2. Coupling

Neti computes per-file coupling:

- afferent coupling (`Ca`): how many files depend on this file
- efferent coupling (`Ce`): how many files this file depends on

Implemented in [src/graph/locality/types.rs](/home/juno/neti/src/graph/locality/types.rs) and [src/graph/locality/coupling.rs](/home/juno/neti/src/graph/locality/coupling.rs).

It also derives:

- instability: `I = Ce / (Ca + Ce)`
- skew: `K = ln((Ca + 1) / (Ce + 1))`

These are used to reason about whether a file behaves like a hub, a leaf, or something unstable.

### 3. Node Identity

Using coupling, Neti classifies each node into a role:

- `STABLE_HUB`
- `VOLATILE_LEAF`
- `DEADWOOD`
- `GOD_MODULE`
- `STANDARD`

Implemented in [src/graph/locality/classifier.rs](/home/juno/neti/src/graph/locality/classifier.rs).

This matters because not all distant dependencies are equally suspicious:

- a stable shared hub can legitimately sit in the middle of the graph
- a random sideways file generally should not

### 4. Cycles

Before applying the rest of the law, Neti checks for dependency cycles.

Implemented in [src/graph/locality/cycles.rs](/home/juno/neti/src/graph/locality/cycles.rs) and enforced in [src/graph/locality/validator.rs](/home/juno/neti/src/graph/locality/validator.rs).

Cycles are treated as foundational architecture failures because they break clean layering and isolate-ability.

### 5. Inferred Layers

If the graph is acyclic, Neti infers architectural layers from dependency order.

Implemented in [src/graph/locality/layers.rs](/home/juno/neti/src/graph/locality/layers.rs).

The implicit rule is:

- dependencies should flow downward into already-established lower layers
- lower layers should not depend upward on higher ones

## The Judgment Model

The validator in [src/graph/locality/validator.rs](/home/juno/neti/src/graph/locality/validator.rs) evaluates edges in this order:

1. detect dependency cycles
2. exempt structural patterns such as legitimate module wiring
3. allow very local edges
4. allow edges within the configured distance budget
5. allow edges that route through stable hubs or configured exemptions
6. reject the remaining edges as locality failures
7. separately classify layer violations such as upward dependencies

This is why the Law of Locality is a composite KPI rather than a single threshold.

It combines:

- physical distance
- graph role
- directional architecture
- structural exemptions

## Violations Neti Emits

Neti currently categorizes locality failures into these named outcomes:

| Code | Meaning |
|---|---|
| `ENCAPSULATION_BREACH` | A file reaches into another module's internal implementation instead of crossing a public boundary |
| `GOD_MODULE` | A file has too many cross-boundary dependencies and behaves like an over-centralized coordinator |
| `MISSING_HUB` | A heavily imported dependency is acting like a shared hub but is not being treated as one |
| `SIDEWAYS_DEP` | A cross-module dependency jumps laterally without acceptable routing |
| `UPWARD_DEP` | A dependency violates inferred architectural layering |

These are categorized in [src/graph/locality/analysis/violations.rs](/home/juno/neti/src/graph/locality/analysis/violations.rs).

## Why This Is Novel

The Law of Locality is not just "imports should be tidy."

It is a Neti-native architectural measure that combines several signals into a single judgment about dependency quality.

That makes it different from:

- raw lint rules
- simple import bans
- one-dimensional coupling metrics

It is closer to an architectural KPI:

- low locality entropy suggests a clean, navigable graph
- high locality entropy suggests boundary decay and rising spaghetti pressure

The current validator tracks an `entropy` value internally as:

`failed_edges / total_edges`

This is implemented in [src/graph/locality/validator.rs](/home/juno/neti/src/graph/locality/validator.rs).

## Design Intent

The Law of Locality exists to enforce a codebase shape where:

- files are understandable in isolation
- dependencies are explainable
- shared concepts are centralized deliberately
- architecture remains navigable as the system grows
- AI-generated code cannot silently accumulate sideways spaghetti

In Neti terms, locality is one of the clearest examples of a proprietary composite metric:

- it is built from lower-level graph and coupling facts
- it expresses an architectural property that users actually care about
- it can be validated empirically against real repositories

## Practical Reading

You can think of the Law of Locality as asking:

- Is this dependency close enough to feel intentional?
- If it is far away, is there a good architectural reason?
- Is this module acting like a shared hub?
- Are boundaries being crossed through the front door or the back door?
- Does the graph still have a sane directional shape?

If the answer keeps being "no," locality is degrading.

## Current Scope

Locality is more language-agnostic than many AST rules because it operates on dependency graphs.

That said, its precision still depends on:

- file discovery
- import extraction quality
- language ecosystem conventions

So the law is conceptually cross-language, while implementation precision varies by language support.
