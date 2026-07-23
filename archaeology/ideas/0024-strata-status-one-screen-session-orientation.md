---
id: ide_01KY7S6GHMQ8ZWNXPX7TX21X7N
sequence: 24
kind: idea
status: parked
created: 2026-07-23
---

# `strata status`: one-screen session orientation

## Problem

CLAUDE.md's archaeology workflow opens with a reorientation ritual:
read the current sprint, inspect pending work, inspect open dragons.
Today that ritual costs several `list` invocations plus file reads, and
every session — human or agent — re-derives it from scratch.
[[dec-concurrent-active-sprints|Decision 15]] widened the question:
"the current sprint" is now a set, so orientation must begin by
discovering which sprints are active at all. Henry's planned
session-start hook wants exactly one deterministic call to answer
"where am I".

## Sketch

`strata status` prints one screen: each active sprint with its pending
tasks, open dragons, the parked-idea count, and doctor's problem count;
`--json` emits the same facts for hooks and automation. Pure
aggregation over facts existing commands already expose — a projection
of current state, not new state and not new semantics.

Relation to [[idea-frontier-projection|frontier projection]]: frontier
ranks actionability and unblocking power across dependency edges;
status merely reports state. Status is the dumb cousin that should
exist first — it supplies the frame that frontier would later refine,
and its output shape is a natural container for frontier's annotations.

## Boundaries

- No recommendations, no ranking, no relevance model.
- Deterministic ordering; output stable across runs on an unchanged
  repository.
- Whole-corpus scanning stays acceptable at current scale per
  [[dec-flat-placement|decision 11]]'s accepted cost; the
  [[ide_01KY5X7C56KBFWJJJKHTEXXQXV|modification watermark]] remains
  the counter-lever if it ever hurts.

## Evidence

The sprint 7 pitch itself reoriented with four `list`/`show`
invocations plus a dozen file reads before work could be proposed.
Sprint 7's task 34 wires a session-start hook out of existing commands;
the friction that hook records in its result is direct evidence for or
against promoting this command.

Proposed by Claude during the sprint 7 pitch, 2026-07-23.
