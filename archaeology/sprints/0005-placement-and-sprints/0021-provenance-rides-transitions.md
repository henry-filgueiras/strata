---
id: tsk-provenance-rides-transitions
sequence: 21
kind: task
status: closed
sprint: spr-placement-and-sprints
created: 2026-07-22
closed: 2026-07-22
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

## Result

`strata close dragon:N --resolved-by <target>` and
`strata adopt idea:N --adopted-by <target>` land the transition and
its provenance edge in one invocation and one atomic write: the edge
target is resolved *before* anything is mutated, so an unresolvable
or vocabulary-illegal target fails the whole command with the
artifact untouched — no transition without its edge, no edge without
its transition. Targets may be stable ids or `kind:N` sequence
references; resolution runs over the whole archaeology harvest, so
`decision:1` reaches the unmanaged decisions collection, and the
bound marker freezes the target's title as its label with YAML
escaping. Ambiguous sequence targets (a dragon 1 collision shape) are
refused naming every candidate. An existing edge is never silently
rewritten — changing recorded provenance stays a deliberate hand
edit. Bare transitions are byte-for-byte unchanged, and the flags are
refused on collections whose vocabulary has no such edge (`close
sprint:N --resolved-by` names the gap). No new edge kinds were
introduced; sprint and task closure gained no provenance surface,
per the decision 10 consumer rule.

The edge-harvest gained sequence and title alongside id and kind to
support write-time binding; `doctor`'s verification universe now
derives from the same harvest, so bind-time and check-time resolution
cannot drift.
