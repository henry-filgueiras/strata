---
id: tsk-fortune-parked-ideas
sequence: 16
kind: task
status: closed
sprint: spr-references-and-ideas
created: 2026-07-22
closed: 2026-07-22
---

# Fortune draws from parked ideas

## Objective

Extend `strata fortune`'s candidate pool to parked ideas, completing
the divergence recorded when idea 6 (`idea-strata-fortune`) was
adopted: "parked ideas join when ideas become a managed collection."
Task 14 removes that gate.

## Acceptance criteria

- The candidate pool is the union of open dragons and parked ideas;
  one draw over the combined set with the existing weight function
  (`max(age_days, 0) + 1`), unchanged.
- Output shape is unchanged; an idea renders with its `idea:N`
  reference, title, age, path, and excerpt exactly as dragons do.
- Adopted and rejected ideas are never selected, alongside the
  existing closed-dragon exclusion.
- The empty-state message accounts for the wider pool (empty means no
  open dragons and no parked ideas).
- Tests extend the existing structure: weight and reachability sweeps
  over mixed pools, membership-of-recall over repeated invocations
  spanning both collections, terminal states never surfacing.
- Idea 6 gains an appended note that the divergence is resolved,
  citing this task.

## Notes

Small and additive by design; if it grows a surprise (it should not),
the surprise is the finding — record it and stop. The `--seed` /
`--json` exclusions from task 8's amendment remain in force.

## Result

As predicted, additive and small: the pool in `main::fortune` is now
open dragons chained with parked ideas, the weight function and draw
untouched, and the empty state names both collections with both
creation commands. Ideas render through the same
reference/title/age/path/excerpt shape because `Summary::reference()`
already carries the kind. No surprises to record. Idea 6 carries the
divergence-resolved update citing this task — the first bound marker
written into an idea artifact.

## Verification

`scripts/check.sh` clean (fmt, 203 tests, clippy). New integration
tests pin the idea output shape, a forty-draw sweep asserting both
collections surface and terminal states never do (miss probability
~2^-39 with equal weights), and the widened empty-state message; the
existing dragon-only tests pass unchanged. Dogfooded live: consecutive
draws in this repository surfaced idea 8, idea 4, idea 10, idea 2,
idea 11, dragon 4, and idea 12 — parked ideas resurface, `doctor`
green throughout.
