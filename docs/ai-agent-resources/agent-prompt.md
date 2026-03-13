# Agent Protocol (SEMMAP-first)

## NON-NEGOTIABLE

STOP: SEMMAP is the controlling workflow in this repo. Do not treat it as optional or supplementary.

STOP: Do not read implementation source before `semmap generate`, `SEMMAP.md` citation, and a declared minimal working set.

STOP: `neti check` is the canonical verification command. Do not use `cargo test` or `cargo clippy` as substitutes.

STOP: Do not leave technical debt in the area you touched.

If SEMMAP/Neti evidence is missing, stop and gather it first.

## No source before orientation

Before reading implementation source beyond the task-defining docs, you MUST:

1. run `semmap generate` for the current repo state,
2. read `SEMMAP.md` and cite the exact line(s) used,
3. run `semmap trace <entry_file>` when flow/ownership/execution path matters,
4. declare a **minimal working set** of 2-5 files, with a short reason for each.

Allowed pre-orientation docs: this prompt, `north-star.md`, `issues-active.md`, `issues-backlog.md`, `issues-done.md`, and relevant `docs/briefs/`.

If you read files beyond the working set, justify why SEMMAP/trace were insufficient.

## Verification

`neti check` is the canonical verification command. It already runs the configured test/clippy/neti suite.

Rules:

* Do NOT use ad hoc `cargo test` or `cargo clippy` as a substitute for `neti check`.
* Only run narrower commands for diagnosis, and say why `neti check` is insufficient for that moment.
* After code changes, the verification result that matters is the exact `neti check` outcome.
* If output is truncated, consult `neti-report.txt`.

## Required evidence per coding iteration

Before edits, provide:

1. **Orientation**: repo purpose, likely entry/trace target, relevant hotspots, short plan.
2. **SEMMAP evidence**: exact cited `SEMMAP.md` lines.
3. **Trace evidence**: exact `semmap trace ...` command(s), when applicable.
4. **Minimal working set**: 2-5 files with reasons.

After edits, provide:

1. key files changed,
2. exact `neti check` result,
3. any manual CLI/UX verification command, important output, and exit code,
4. issue-file updates made.

If you cannot provide this evidence, stop and run the missing SEMMAP/Neti steps first.

## Workflow

1. Run `semmap generate`; read `SEMMAP.md`, `north-star.md`, and active issues.
2. Write Orientation.
3. Run `semmap trace <entry_file>` when needed.
4. Declare the minimal working set, then read only those files. Prefer `semmap cat` when practical.
5. Make minimal edits that respect SEMMAP boundaries. Hotspots mean smaller diffs and stronger tests.
6. Run `neti check`; iterate until clean or until only clearly pre-existing failures remain and are explicitly called out.
7. Resolve technical debt in the area you touch; do not say "I didn't break it so I'm leaving it broken."

## Issue discipline

Work from `docs/issues/issues-active.md` first, then `docs/issues/issues-backlog.md`.

An issue is DONE only when verification proves it. When relevant, that means fail-before/pass-after and at least one real edge case.

For every touched issue:

* keep `**Status:**`, `**Labels:**, and **Resolution:**,
* keep `**Files:**` aligned with the real implementation surface,
* add/fix dependency references when they matter,
* move finished work into the correct issue file.

When you complete or materially refine an issue, update `**Resolution:**` with:

* what changed,
* why this approach was chosen,
* how it was verified,
* which commands were run,
* whether any remaining failures are pre-existing.

## Environment guidance

* On Windows, do not get stuck retrying fragile patch flows; use the most reliable edit method.
* If a tool keeps failing, switch approaches quickly.
* When evaluating a freshly cloned external repo, expect `semmap generate` may need `--purpose`; rerun with a concise, honest purpose immediately.

## Minimal close-out

A compliant final report for code work should usually include:

1. issue handled,
2. SEMMAP evidence used,
3. key files changed,
4. exact `neti check` outcome,
5. any manual verification performed,
6. whether issue records were updated.
