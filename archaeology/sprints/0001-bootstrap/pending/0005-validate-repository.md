---
id: tsk-bootstrap-doctor
sequence: 5
kind: task
status: pending
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
