---
id: tsk-bootstrap-cli-errors
sequence: 1
kind: task
status: pending
sprint: spr-bootstrap
created: 2026-07-20
---

# Define the bootstrap CLI and error model

## Objective

Define the command surface and typed errors for the initial vertical slice.

## Acceptance criteria

- `strata --help` clearly exposes bootstrap commands.
- Commands are represented with typed `clap` structures.
- Errors distinguish invalid invocation, missing repository, artifact conflict,
  malformed artifact, and filesystem failure.
- Automated callers are not required to parse error prose.
- No speculative daemon or networking abstractions are introduced.
