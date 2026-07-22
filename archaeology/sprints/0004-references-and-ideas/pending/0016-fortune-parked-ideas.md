---
id: tsk-fortune-parked-ideas
sequence: 16
kind: task
status: pending
sprint: spr-references-and-ideas
created: 2026-07-22
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
