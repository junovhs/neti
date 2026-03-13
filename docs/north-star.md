# Model-Driven Code Intelligence: North Star

## The Problem

AI makes producing code cheap. That doesn't eliminate work — it changes its shape.

When diffs are cheap, more experiments ship, more partial ideas land, and the codebase grows faster than shared understanding. Patches compile and pass tests, but boundaries erode, coupling rises, and the system becomes harder to change safely. Call this **diff inflation**.

At the same time, cheap implementation removes a forcing function that used to improve judgment. When shipping was expensive, teams were forced to prioritize and kill mediocre ideas early. When shipping is cheap, you accumulate features that don't move outcomes and become permanent maintenance cost.

The bottleneck moved: from *writing code* to **preserving coherence and judgment under rapid change.**

## The Goal

**High-velocity change without structural decay or declining product judgment.**

## The Three Tools

**Semmap** is the comprehension engine. It compresses the repo into a high-signal map — layers, responsibilities, hotspots, execution spines — so an AI can orient before it touches anything. You don't hand over the codebase; you hand over the map.

**Neti** is the physics engine. It enforces structural constraints after code is generated — dependency rules, complexity budgets, cycle bans, safety checks. Passing tests is necessary but not sufficient; structure must remain sound.

**Ishoo** is the judgment layer. It tracks what needs doing and why. It's the forcing function that asks "is this worth building" before the work starts, and "did this actually ship" when it closes.

Together: orient → code → verify → close. Each tool owns one part of that loop and nothing else.

## The Alignment Test

When a feature idea comes up for any of the three tools, ask:

**Does this make orientation faster, enforcement stricter, or judgment clearer?**

If it doesn't do one of those three things, it's scope creep regardless of how useful it feels.

Some examples of things that pass:
- Semmap surfaces hotspots earlier in the map → faster orientation
- Neti catches a new class of architectural violation → stricter enforcement  
- Ishoo shows which issues are blocked vs. ready → clearer judgment

Some examples of things that fail:
- Semmap generates a "brief" document that the agent could assemble itself
- Neti adds a style linter → that's a different tool's job
- Ishoo adds a time-tracking feature → interesting, but not judgment

## What "Model-Driven" Means

The model is a small, explicit, machine-checkable description of the system's structure: which modules own which responsibilities, what may depend on what, where execution starts, and what numeric limits apply.

If the model is not kept current, it becomes fiction. So it must be versioned alongside the code and updated when structure changes. Model drift is not docs debt — it's correctness debt.

Natural language constraints drift. The only constraints that survive diff inflation are computable ones.
