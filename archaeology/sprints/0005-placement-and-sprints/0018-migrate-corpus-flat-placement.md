---
id: tsk-migrate-corpus-flat-placement
sequence: 18
kind: task
status: closed
sprint: spr-placement-and-sprints
created: 2026-07-22
closed: 2026-07-22
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

## Result

Corpus and code crossed to flat placement in one slice; `doctor` is
green and every check passes. The `git mv` sweep moved 24 files —
dragons, ideas, and every sprint's tasks including this file — with no
byte of content changed, and the retired lifecycle directories are
gone.

Code alignment turned out to be a deletion job, confirming the
decision's premise:

- `read::Collection` lost its `(status, dir)` map for a single `dir`
  plus a state list; `dir_of` is gone.
- The transition module halved: the two-step contract, its rollback
  path, the `TransitionFs` fault-injection trait, and the
  `transition-interrupted` error (exit code 10, now documented as
  retired, not reused) all fell out. A transition is one safe write.
- The read pipeline's lifecycle-mismatch check — the failure class the
  decision retires — was deleted rather than rewritten; its successor
  is the stray-directory conflict (a directory inside a collection
  directory is an error-tier finding naming decision 11), covered in
  both strict scans and `doctor`.
- Tests pinning the old model were repurposed, not padded: the
  crash-window and destination-collision specimens are inexpressible
  now, and their replacements pin in-place rewrites, terminal-state
  refusals, and the leftover-subdirectory finding.

One behavioral note for the record: `list` and `show` output now
carries the shorter flat paths, and sequence-tiebreak ordering across
duplicate sequences is by filename within one directory instead of by
lifecycle directory name — both covered by updated tests. README's
sample output and layout tree are updated; `bootstrap-inception.sh`
deliberately is not (it reproduces the historical seed state and says
so).
