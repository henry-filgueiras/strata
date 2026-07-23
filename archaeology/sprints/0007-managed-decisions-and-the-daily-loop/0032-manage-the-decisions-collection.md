---
id: tsk_01KY7S6Q7VF46RFDNFTCY9E5B2
sequence: 32
kind: task
status: pending
sprint: spr_01KY7S6Q69YJ6HATZB48SZBRRM
created: 2026-07-23
---

# Manage the decisions collection

## Objective

Make decisions the fifth managed collection: creation, listing,
showing, and doctor coverage over the existing fifteen-file corpus,
without modifying that corpus. If the implementation reveals that the
fifth collection requires a wholesale further copy of collection
mechanics, stop and surface [[idea-declarative-collection-specs|idea
10]] as a decision point before proceeding — neither copy nor extract
silently.

## Acceptance criteria

- `strata new decision "<title>"` creates a correctly sequenced,
  slugged, ULID-identified decision in `archaeology/decisions/` with
  status `accepted` and a Context / Decision / Consequences scaffold.
- `strata list decisions` and `strata list decisions --json` list the
  full corpus in sequence order; `strata show decision:N` and its
  `--json` form work for both legacy-id and ULID decisions.
- `doctor` applies the same structural invariants to decisions as to
  the other managed collections and stays green on the unmodified
  existing corpus.
- No existing decision file changes in this task.
- Temp-directory tests cover creation, discovery, sequence allocation,
  and malformed metadata for decisions; `scripts/check.sh` passes.
