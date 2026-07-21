---
id: tsk-bootstrap-doctor
sequence: 5
kind: task
status: closed
sprint: spr-bootstrap
created: 2026-07-20
---

# Validate repository invariants

## Objective

Implement `strata doctor` for the bootstrap artifact model.

## Acceptance criteria

Detect and report:

- malformed front matter;
- metadata inconsistent with file placement;
- duplicate stable identities;
- duplicate display sequences;
- invalid filenames;
- unreadable files.

Validation must not modify canonical files during this sprint.

## Result (2026-07-21)

Implemented as `src/doctor.rs`, reusing the read-side per-file parse
pipeline so validation semantics cannot drift between `list`/`show` and
`doctor`. All acceptance criteria are covered by unit and integration
tests: malformed front matter, placement mismatches, duplicate stable
ids, duplicate display sequences, invalid filenames, and unreadable
files are collected as findings — the scan never stops at the first
problem and never mutates canonical files.

Prerequisite: dragon 2 was resolved first (decision 0005 update,
2026-07-21) so doctor does not classify Git-inevitable states as
corruption; a marker-only repository is healthy.

Provisional calls, made to unblock the workflow and recorded as
learning-experience decisions rather than settled contracts (see the
2026-07-21 update to decision 0004):

- outcome contract: findings on stdout (human lines or `--json` array),
  summary via `error[unhealthy-repository]` exit 9 on stderr;
- finding vocabulary: five `problem` codes, one finding per duplicate
  group anchored at its first path;
- scope: doctor validates what Strata manages (the dragon collection
  plus managed-path conflicts); manually maintained collections are out
  of scope until collections generalize.

Dogfooded: `strata doctor` reports this repository healthy, including
the exit-0 and `--json` paths.
