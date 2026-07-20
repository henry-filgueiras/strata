---
id: dec-bootstrap-error-contract
sequence: 4
kind: decision
status: accepted
created: 2026-07-20
---

# Machine-readable error contract

## Context

Automated callers must not parse human prose to distinguish failure modes.
The bootstrap needed a contract that works before `--json` output exists and
remains valid after it arrives.

## Decision

Strata failures expose exactly two stable machine surfaces:

- the process exit code:
  - 0 success
  - 1 transitional (`unimplemented` stubs; general failure)
  - 2 invalid invocation (shared with `clap` usage errors)
  - 3 missing repository
  - 4 artifact conflict
  - 5 malformed artifact
  - 6 filesystem failure
- a leading `error[<code>]:` token on the first stderr line, where `<code>`
  is a stable kebab-case category identifier (`invalid-invocation`,
  `missing-repository`, `artifact-conflict`, `malformed-artifact`,
  `filesystem-failure`, `unimplemented`).

Message text after the colon is human-oriented and may change freely. It
should name the operation, the artifact or path, the violated invariant, and
a next step.

## Consequences

- Exit codes and category identifiers are compatibility surfaces; changing
  them is a breaking change requiring a new decision.
- The `unimplemented` category is transitional scaffolding and is removed as
  bootstrap commands gain behavior.
- Future `--json` error output may add detail but must keep these categories
  consistent.
