---
id: tsk_01KY7S6Q9FVH11FYN5VM9VBE13
sequence: 33
kind: task
status: pending
sprint: spr_01KY7S6Q69YJ6HATZB48SZBRRM
created: 2026-07-23
---

# Run doctor in the check gate

## Objective

Make archaeology validity a commit-gate fact: `scripts/check.sh` runs
`strata doctor` against this repository and fails when doctor reports
problems, so no commit that satisfies the gate can carry a corrupted
corpus.

## Acceptance criteria

- `strata doctor` exits nonzero when it reports problems — confirmed
  by existing behavior or added, with a test, in this task.
- `scripts/check.sh` invokes doctor against this repository and fails
  when doctor fails.
- The gate is demonstrated once against a deliberately corrupted
  scratch copy, and the evidence is recorded in this task's result.
- The added wall-clock cost of the step is noted in the result.
