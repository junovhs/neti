# Locality v2: Antifragile Architecture Detection

**Status:** Approved Proposal  
**Created:** 2025-12-28  
**Priority:** Next implementation target

---

## Executive Summary

The current locality system requires manual configuration to avoid false positives. This proposal upgrades it to be zero-config and self-correcting by:

1. Detecting dependency cycles (hard error)
2. Auto-detecting hubs from graph metrics
3. Only flagging bidirectional coupling (not layered dependencies)
4. Inferring architectural layers automatically

The result: zero config, zero false positives, genuine architectural enforcement.

---

## 1. Problem Statement

### Current Behavior

The locality scanner produces violations that require manual intervention:

```
? MISSING_HUB
    src\clipboard\mod.rs (fan-in: 4)
     Add to [rules.locality].hubs in slopchop.toml

? TIGHT_COUPLING
    src\cli\handlers.rs  src\apply\mod.rs
     'cli'  'apply' coupled. Merge or extract shared interface
```

### Why This Is Fragile

1. **Manual hub declarations decay.** Every new utility module requires config updates.

2. **False positive tight coupling.** `cli  apply` is correct layering, not bad coupling. The algorithm cannot distinguish intended architecture from accidental spaghetti.

3. **Configuration as debt.** The more config required, the less the tool gets used.

### The Antifragile Principle

A system is antifragile when it gets *better* under stress. An antifragile locality system would:

- Require zero configuration
- Automatically adapt as the codebase evolves
- Only flag genuine architectural violations
- Never produce false positives for standard patterns

---

## 2. The Core Insight: No Cycles

The Acyclic Dependencies Principle (ADP) is one of the most agreed-upon rules in software architecture:

> "The dependency graph of packages must have no cycles." - Robert C. Martin

**Why cycles are universally bad:**

1. **Can't understand in isolation** - To understand A, you need B. To understand B, you need A.
2. **Can't test in isolation** - Same problem.
3. **Can't compile separately** - Everything recompiles together.
4. **Indicates confused responsibilities** - If A needs B and B needs A, they're one thing pretending to be two.

**Languages that enforce this:**
- **Rust:** Circular mod imports are a compile error
- **Go:** Circular imports are a compile error
- **Java/C#:** Allow cycles  every large codebase becomes spaghetti

**The insight:** If we enforce no cycles, layer inference becomes trivial. A DAG (directed acyclic graph) can always be topologically sorted into layers. Cycles are the only thing that breaks it.

**Two birds, one stone:** Cycle detection is both a critical architectural check AND the foundation for layer inference.

---

## 3. Current Architecture

### What Exists

```
src/graph/locality/
��� classifier.rs    # Computes node identity (Hub, Leaf, Deadwood, GodModule)
��� coupling.rs      # Measures cross-module coupling
��� distance.rs      # LCA-based topological distance
��� validator.rs     # Universal Locality Algorithm
��� mod.rs
```

### Current Config (Fragile)

```toml
[rules.locality]
max_distance = 4
hub_threshold = 1.0
min_hub_afferent = 5
hubs = ["src/clipboard/mod.rs", "src/stage/mod.rs"]  # manual babysitting
```

---

## 4. Proposed Solution

### Phase 0: Cycle Detection (Foundation)

**The rule:** No dependency cycles. Ever.

```rust
fn detect_cycles(graph: &Graph) -> Vec<Cycle> {
    // Standard DFS-based cycle detection
    // Returns all cycles found as ordered lists of modules
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    let mut cycles = Vec::new();
    
    for module in graph.modules() {
        if !visited.contains(&module) {
            dfs_find_cycles(module, graph, &mut visited, &mut rec_stack, &mut cycles);
        }
    }
    
    cycles
}
```

**Output:**
```
DEPENDENCY CYCLE DETECTED (hard error)

  apply/mod.rs
     cli/handlers.rs
     apply/verification.rs
     apply/mod.rs   cycle completes here

Cycles indicate confused responsibilities. 
Either merge these modules or extract shared logic to a lower layer.
```

**This is a hard error.** Cycles block all other locality checks. Fix cycles first.

---

### Phase 1: Auto-Hub Detection

**Change:** Remove manual `hubs` config. Compute hub status from graph metrics.

```rust
fn is_hub(node: &Node, graph: &Graph) -> bool {
    let fan_in = graph.afferent_coupling(node);
    let fan_out = graph.efferent_coupling(node);
    
    // High fan-in, low fan-out = hub (utility that many depend on)
    fan_in >= 3 && fan_in > fan_out
}
```

**Result:** `clipboard` and `stage` automatically become hubs. No config needed. New utility modules auto-promote as usage grows.

---

### Phase 2: Directional Coupling Detection

**Change:** Only flag bidirectional coupling, not unidirectional layering.

```rust
fn detect_coupling(graph: &Graph) -> Vec<CouplingViolation> {
    let mut violations = Vec::new();
    
    for (module_a, module_b) in graph.module_pairs() {
        let a_to_b = graph.has_edge(module_a, module_b);
        let b_to_a = graph.has_edge(module_b, module_a);
        
        if a_to_b && b_to_a {
            // Bidirectional = actual coupling problem
            violations.push(CouplingViolation::Bidirectional { a: module_a, b: module_b });
        }
        // Unidirectional is fine - that's just layering
    }
    
    violations
}
```

**Result:** `cli  apply` stops being flagged. Only genuine mutual dependencies get reported.

---

### Phase 3: Layer Inference

**Prerequisite:** No cycles (Phase 0 must pass)

**Algorithm:**

```rust
fn infer_layers(graph: &Graph) -> Vec<Vec<Module>> {
    let mut layers = vec![];
    let mut assigned = HashSet::new();
    
    loop {
        // Find modules whose dependencies are all in lower layers
        let next_layer: Vec<_> = graph.modules()
            .filter(|m| !assigned.contains(m))
            .filter(|m| graph.deps(m).all(|d| assigned.contains(&d)))
            .collect();
        
        if next_layer.is_empty() { break; }
        
        for m in &next_layer { assigned.insert(m.clone()); }
        layers.push(next_layer);
    }
    
    layers
}
```

**Output:**
```
INFERRED LAYERS

  L0 (foundation):  constants, error, tokens
  L1 (config):      config, types
  L2 (utilities):   clipboard, stage, skeleton, lang
  L3 (core):        discovery, analysis, graph
  L4 (features):    apply, pack, audit, signatures, map
  L5 (interface):   cli
  L6 (entry):       bin

All dependencies flow downward. �
```

**Violation rule:** Edges must go down. Layer N can import from N-1, N-2, etc. Never from N+1.

---

## 5. Configuration After Changes

### Before (Fragile)

```toml
[rules.locality]
max_distance = 4
hub_threshold = 1.0
min_hub_afferent = 5
hubs = [
    "src/clipboard/mod.rs",
    "src/stage/mod.rs",
]
```

### After (Antifragile)

```toml
[rules.locality]
mode = "warn"   # or "error"
```

That's it. Everything else is inferred from the graph.

---

## 6. Implementation Plan

| Phase | Effort | Risk | Description |
|-------|--------|------|-------------|
| 0 | 1-2 hrs | Low | Cycle detection (DFS, hard error) |
| 1 | 1-2 hrs | Low | Auto-hub (fan-in threshold) |
| 2 | 2-3 hrs | Low | Directional coupling (bidirectional only) |
| 3 | 2-3 hrs | Medium | Layer inference (toposort + violation check) |

**Total:** 6-10 hours

**File locations:**
- Phase 0: New `src/graph/locality/cycles.rs`
- Phase 1: Modify `src/graph/locality/classifier.rs`
- Phase 2: Modify `src/graph/locality/coupling.rs`
- Phase 3: New `src/graph/locality/layers.rs`

---

## 7. Success Criteria

| Metric | Before | After |
|--------|--------|-------|
| Config lines required | 5-10+ | 1 |
| False positives on SlopChop | 7 | 0 |
| Manual hub declarations | Required | Never |
| Cycle detection | None | Hard error |

**Definition of Done:**

1. `slopchop scan --locality` on SlopChop produces 0 false positives
2. Cycles are detected and reported as hard errors
3. No `hubs = [...]` in config
4. TIGHT_COUPLING only fires for bidirectional dependencies
5. Layer visualization shows correct architecture

---

## 8. Expected Output After Implementation

```
$ slopchop scan --locality

LOCALITY SCAN

Cycle Check .............. � No cycles detected
Hub Detection ............ 3 auto-detected (clipboard, stage, config)
Layer Analysis ........... 7 layers inferred

LAYERS
  L0: constants, error, tokens
  L1: config, types  
  L2: clipboard, stage, skeleton, lang
  L3: discovery, analysis, graph
  L4: apply, pack, audit, signatures, map
  L5: cli
  L6: bin

EDGES: 144 total | 144 passed | 0 violations

Topological Entropy: 4.9%
Architecture is clean.
```

---

## 9. Instructions for Implementation

**To the next AI session:**

Your top priority is implementing Phases 0-3 of this document.

1. Read this entire proposal first
2. Implement in order: Phase 0  1  2  3
3. Test each phase with `slopchop scan --locality` before proceeding
4. Success = zero false positives on SlopChop itself
5. Update docs/past-present-future.md when complete