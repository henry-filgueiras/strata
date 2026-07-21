---
id: idea-frontier-projection
sequence: 8
kind: idea
status: parked
created: 2026-07-21
---

# Frontier projection: what is actionable now, and what unblocks the most

## Problem

"What's next?" is answered today by a human or agent re-deriving the
dependency structure by hand: reading the sprint, the pending tasks, and
the dragons whose resolution criteria gate them. The 2026-07 sequencing
of dragon 2 → doctor → fortune → chores was exactly this manual
topological sort. The information is already in the repository; only the
traversal is missing.

## Sketch

A read-only projection — `strata next` or similar — that walks typed
dependency edges among pending tasks in open sprints and presents the
*frontier*: nodes with no unresolved hard prerequisite. Optional ranking
layers on top: shortest path to a designated milestone or high-priority
output, or highest out-degree ("unblocks the most"), in the spirit of
critical-path method.

Two boundaries keep it honest. It is a disposable projection over
canonical edges — it computes, never mutates, and its ranking is advice.
And it deliberately stops short of the recorded non-goal of autonomous
task selection: the frontier is *presented*; choosing remains with the
caller. The gap between "here is what is unblocked" and "I have decided
what to do" is load-bearing.

Blocked on dragon 3 (`drg_01KY169X7W0YXJ5QFV4D1MK4FB`): without a
machine-parseable reference syntax and typed edge vocabulary (a
`blocked-by` / `depends-on` kind with DAG semantics), there is no graph
to walk — the same dependency that gates
[[idea-doctor-reference-graph]]. Cycle detection in that vocabulary
would be a doctor concern; this projection assumes a valid DAG.

## Evidence

Motivating instance: the doctor-scoping conversation (2026-07-21), where
the frontier had to be re-derived by reading four artifacts. Decision
0006 (`dec-bootstrap-reference-model`) supplies edge semantics; CLAUDE.md
lists autonomous task selection as an explicit non-goal, which this idea
respects by presenting rather than choosing. Prior art: `make` and every
DAG scheduler, critical-path method, "next action" lists in GTD.
