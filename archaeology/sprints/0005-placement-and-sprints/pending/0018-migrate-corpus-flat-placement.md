---
id: tsk-migrate-corpus-flat-placement
sequence: 18
kind: task
status: pending
sprint: spr-placement-and-sprints
created: 2026-07-22
---

# Migrate the corpus and code to the decided placement model

## Objective

Apply task 17's decision to the repository and the tool in one
reviewable slice: move every artifact to its decided location with
history preserved, align dragon and idea discovery, transition, and
validation code with the new model, and leave `doctor` green.

## Acceptance criteria

- Every existing artifact sits at its decided path, moved with
  `git mv`; no stable identity, sequence, or file content other than
  paths changes, per [[dec-bootstrap-stable-identity|decision 2]].
- Dragon and idea code discovers artifacts under the new layout;
  transitions rewrite status in place without moving files, under the
  decision 8 failure classes as amended by task 17.
- `doctor` enforces the new model: the status/placement agreement
  check is replaced per the decision, malformed or unknown `status`
  values remain errors, and the full suite of existing checks still
  passes on this repository.
- Sprint 5's own artifacts (this file included) are swept by the same
  migration, and their `status` fields remain the lifecycle authority.
- Tests covering discovery, transitions, and doctor are updated to
  the new layout; `scripts/check.sh` passes.
- The old lifecycle directories are gone; no code path or document
  still references them except historical records.
