---
id: idea-typed-dragon-resolution-edges
sequence: 5
kind: idea
status: parked
created: 2026-07-20
---

# Typed resolution edges between dragons and decisions

## Problem

Dragons record how they expect to be resolved as prose promises ("record
the outcome as an update to decision 0005"), and closed dragons will carry
no machine-checkable pointer to whatever settled them. Promises can dangle
silently forever.

## Sketch

A `resolved-by` typed edge on closed dragons pointing at the resolving
decision or task, plus the inverse expectation that doctor can verify:
every closed dragon carries the edge, and every edge target exists. Part
of the initial typed vocabulary candidates (supersedes, amends,
resolved-by, implements, context-from) enumerated in dragon 3.

## Evidence

Dragon 3 (`drg_01KY169X7W0YXJ5QFV4D1MK4FB`); the dangling-promise gap
observed when dragon 2 pledged its outcome to decision 0005 with nothing
able to check it; idea 2 (`idea-doctor-reference-graph`) as the consumer
that would enforce it.
