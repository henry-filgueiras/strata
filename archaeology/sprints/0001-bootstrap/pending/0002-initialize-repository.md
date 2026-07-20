---
id: tsk-bootstrap-init
sequence: 2
kind: task
status: pending
sprint: spr-bootstrap
created: 2026-07-20
---

# Initialize a Strata repository

## Objective

Implement `strata init` for the smallest supported repository layout.

## Acceptance criteria

- Creates required directories and configuration safely.
- Refuses to overwrite conflicting existing files.
- Re-running against an already valid repository is non-destructive.
- Works in a temporary non-Git directory.
- Partial failures do not leave truncated files.
