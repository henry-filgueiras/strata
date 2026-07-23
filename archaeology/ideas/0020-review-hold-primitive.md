---
id: ide_01KY64ZPXVR0XRZBHKERBXXJ0C
sequence: 20
kind: idea
status: parked
created: 2026-07-22
---

# Review-hold primitive

## Problem

The sprint 5 stop-the-line hold (comment thread 3) is enforced by an
accident: sprint 6 is active, and the disputed single-active-sprint
rule makes that block `new sprint`. Review gating and sprint
concurrency are different axes — thread 3 disclaims the mechanism
explicitly, and thread 7's claim A disputes the singleton itself. If
task 28 confirms concurrent sprints, nothing mechanical prevents
opening a feature sprint past open blocking review threads; the hold
reverts to convention.

## Sketch

A first-class, narrowly scoped hold: gate-relevant intent commands
(`new sprint` at minimum) consult open blocking review threads — or a
dedicated hold artifact — and refuse with the blocking thread named
and the way out (repair, rejection with evidence, or owner waiver, per
thread 3's exit conditions). Orthogonal to cardinality: a hold blocks
because review demands it, not because a sprint slot is occupied.
Lifting is an explicit operation with a recorded reason, never a side
effect of closing whichever sprint happens to host remediation.
Depends on comment threads becoming a managed or at least parseable
collection ([[idea-comment-threads|idea 11]]); the `review.gate`
front matter in the existing specimens is the natural seed.

## Evidence

Motivating instance: sprint 6's rationale explicitly borrows the
singleton as its "mechanical interlock" (2026-07-22), while thread 3
disclaims that mechanism as evidence for the policy and thread 7
alleges the policy is unadjudicated — the hold currently stands on the
disputed rule. Prior art: branch protection gates merges independently
of how many branches exist; an andon cord stops the line without
dictating how many lines the factory runs. Related:
[[ide_01KY5YG15T64AA6K5F0VVDJT97|idea 19]] supplies the review
ceremony whose threads a hold would consult.
