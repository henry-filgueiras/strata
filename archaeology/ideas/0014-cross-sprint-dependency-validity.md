---
id: idea-cross-sprint-dependency-validity
sequence: 14
kind: idea
status: parked
created: 2026-07-22
---

# Concurrent sprints and cross-sprint dependency validity

## Problem

Sprint 3 (`spr-community-standards`) opened while sprint 2 was still
active, and nothing in the framework said whether that is legal: the
workflow prose assumes a singular "current sprint," but no invariant
forbids concurrency, and disjoint concurrent sprints proved
unproblematic in practice. When sprints become a managed collection, a
future `strata sprint` command and `doctor` need a definite answer —
and the interesting failure mode is not concurrency itself but
*coupling*: a sprint whose plan silently depends on another active
sprint's unlanded work.

## Sketch

Concurrent active sprints are valid, and for disjoint work possibly
preferred (owner position, 2026-07-22 discussion). Validity is governed
by two rules of different strengths:

- **Deterministic**: a hard dependency edge that crosses a sprint
  boundary must target settled work — a terminal-state task or a
  timeless canonical artifact such as a decision. Pending work
  depending on another sprint's pending work is a structural failure:
  refused at intent-command time (sprint open, task creation, edge
  addition), diagnosed by `doctor` when hand-edits produce it. The
  rationale: a sprint is the unit of commitment, and a cross-sprint
  edge into unlanded work couples two commitments so that one's slip
  silently invalidates the other's plan. The refusal is actionable —
  move the dependent task into that sprint, or sequence the sprints.
  Note the rule needs no "concurrent" special case: sequential sprints
  depending on prior sprints' closed work (sprint 2 on sprint 1's
  outputs) satisfy it trivially.
- **Advisory**: a sprint that opens with an *empty frontier* — every
  task gated on an unresolved decision or dragon — warns but does not
  fail. Resolving blockers mid-sprint is normal, evidenced by sprint 2
  shipping task 7 gated on a contract question that decision 8 settled
  mid-flight. The honest diagnostic is "nothing is actionable at
  birth," not "your dependencies are illegitimate."

Intra-sprint pending-on-pending dependencies remain plain ordering —
the frontier material [[idea-frontier-projection]] walks — never a
finding. The two-tier split lands on the existing structural-versus-
advisory boundary ("semantic systems advise; they do not define
truth").

Both checks compute from the same typed-edge graph, so nothing here is
implementable before dragon 3 (`drg_01KY169X7W0YXJ5QFV4D1MK4FB`)
lands. Dragon 3 requires each edge kind to define its doctor semantics
up front; this rule is precisely that definition for the future
`depends-on` / `blocked-by` kind, and this idea is a consumer of that
vocabulary, not a proposal for new syntax.

## Evidence

Motivating instance: sprint 3 running concurrently with sprint 2
(2026-07-22), disjoint scope, declared cost-free — with the validity
question recorded in its rationale and settled in direction by owner
review the same day. Existing composition rules already constrain the
edge graph (ideas are never load-bearing, so no dependency may target
one). Prior art: DAG schedulers refuse edges into nodes that do not
exist yet; kanban WIP limits treat concurrent commitment, not
concurrent activity, as the risk; `make` treats a missing prerequisite
as failure but an out-of-date one as work.
