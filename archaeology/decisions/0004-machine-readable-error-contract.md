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

## Update (2026-07-20): read-side resolution categories

Task 0004 (list/show) appended two categories for reference resolution;
existing codes and numeric assignments are unchanged:

- exit code 7, category `artifact-not-found`: no managed artifact matches
  the requested stable ID or `collection:sequence` reference;
- exit code 8, category `ambiguous-reference`: a reference that must name
  exactly one artifact matches several (duplicate display sequences or
  duplicate stable IDs); Strata never silently picks one, and the message
  names every candidate path.

Success-path `--json` output added by the same task is a further
compatibility surface: field names, field order, and deterministic
ordering of `list`/`show` projections are pinned by tests. Errors remain
stderr-only, so `--json` stdout stays parseable on every failure.

## Update (2026-07-21): doctor outcome category; `unimplemented` retired

Task 0005 (`doctor`) appended one category and retired one:

- exit code 9, category `unhealthy-repository`: `doctor` completed its
  scan and the repository has validation findings. The findings are the
  stdout payload — human lines or a deterministic `--json` array of
  `{problem, path, detail}` objects — while stderr carries only the
  summary token line, preserving the errors-are-stderr-only rule.
- the transitional `unimplemented` category is removed with the last
  stub: every bootstrap command now has behavior. Exit code 1 stays
  reserved for general failure and is not reassigned.

The finding `problem` codes (`malformed-artifact`, `unreadable-artifact`,
`artifact-conflict`, `duplicate-id`, `duplicate-sequence`) are a
provisional vocabulary, deliberately recorded as a learning-experience
call rather than a settled contract: one collection and one validation
pass are not enough evidence to freeze finding taxonomy. Revisit when
doctor grows beyond the dragon collection or gains reference-graph
checks (idea 2); exit code 9 and the category identifier itself are
stable surfaces like the rest of this decision.
