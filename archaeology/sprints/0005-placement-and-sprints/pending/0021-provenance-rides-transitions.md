---
id: tsk-provenance-rides-transitions
sequence: 21
kind: task
status: pending
sprint: spr-placement-and-sprints
created: 2026-07-22
---

# Provenance rides the transition commands

## Objective

Land the companion piece decision 10 observed: terminal transitions
accept their provenance edge in the same invocation, so recording why
something closed no longer means hand-editing front matter after the
tool has already moved on. No new edge kinds — only a command surface
for the vocabulary decision 10 already fixed.

## Acceptance criteria

- `strata close dragon:N --resolved-by <target>` performs the
  transition and writes the `resolved-by` edge in one invocation; the
  edge value is a quoted bound marker per decision 10, so a
  sequence-form target is resolved to its stable ID and label at write
  time.
- `strata adopt idea:N --adopted-by <target>` does the same for
  `adopted-by`.
- A target that does not resolve to an existing artifact fails the
  whole invocation under the decision 8 failure classes; no transition
  without its edge, no edge without its transition.
- The flags are optional; bare transitions behave exactly as before.
- `doctor`'s existing typed-edge verification covers the written
  edges with no new check code beyond what the flags require.
- If sprint or task closure grows a provenance flag here, it is only
  because a consumer exists in this sprint; otherwise that surface
  waits, per the decision 10 rule.
