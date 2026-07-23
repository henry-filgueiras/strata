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

## Incident evidence: the hold ran to completion (2026-07-23)

The Problem and Sketch above were written mid-incident; this section
records how the incident actually closed (2026-07-22) as executable
evidence for a first-class review interlock.

- Sprint 5 reached `main` before its intended review was complete;
  [[spr_01KY61D615FAC8VVSTD7QXX1DW|sprint 6]] then served as a
  temporary remediation interlock while the stop-the-line incident
  was adjudicated.
- Umbrella [[cmt-sprint5-post-merge-stop-the-line|thread 3]] carried
  blocking-gate metadata (`review.gate: blocks-new-sprint`) until all
  child findings were disposed, and every accepted finding received
  an explicit owner and executable verification before the hold was
  released through thread 3's eight closure conditions.
- The mechanism worked socially and archaeologically — nothing
  slipped past the gate — but it depended on manual convention, not
  an enforceable repository state. Once decision 15 legalized
  concurrent sprints and removed the accidental single-sprint mutex,
  the hold stood on thread 3's procedural authority alone: nothing
  mechanical would have refused a feature sprint opened past it.
- The lesson is not that all work requires bureaucracy. It is that
  some named review campaigns need a durable, queryable gate whose
  release conditions cannot disappear into chat or operator memory.

Distinct states the incident proved must not be conflated: an active
sprint, an unresolved blocking review, ordinary pending work, and a
release/merge prohibition are four different things — the incident
held all four at once and only convention kept them apart. A hold
primitive must not assume they share one lifecycle state.

**Present disposition:** parked, unchanged.

**Next investigation (bounded):** specify the smallest review-hold
primitive that can answer: what is held; why it is held; who or what
owns each exit condition; whether new work may continue concurrently;
what evidence releases the hold; and which commands and doctor
findings expose violations.

**Excluded for now:** Git hooks, GitHub branch protection, CI policy,
and any generalized workflow engine — possible enforcement adapters,
not yet the domain model.
