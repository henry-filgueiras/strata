---
id: tsk-bootstrap-list-show
sequence: 4
kind: task
status: pending
sprint: spr-bootstrap
created: 2026-07-20
---

# List and show artifacts

## Objective

Rediscover artifacts from canonical files and expose human and machine
projections.

## Acceptance criteria

- `strata list dragons` produces concise human-readable output.
- `strata list dragons --json` produces deterministic structured output.
- `strata show` can resolve a stable identity or unambiguous human reference.
- Malformed files are reported rather than silently skipped.
- JSON field names are documented by tests.
