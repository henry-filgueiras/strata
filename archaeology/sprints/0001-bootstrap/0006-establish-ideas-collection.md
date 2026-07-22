---
id: tsk-bootstrap-ideas-collection
sequence: 6
kind: task
status: closed
sprint: spr-bootstrap
created: 2026-07-20
closed: 2026-07-20
---

# Establish the ideas collection

## Objective

Give parked future ideas a first-class, individually addressable home so
logs stop standing in for a backlog (gap identified in log 0002).

This is manual-layout and convention work only; Strata CLI support for an
`idea` collection is out of bootstrap scope.

## Acceptance criteria

- `archaeology/ideas/` exists, following the four-digit sequence and
  kebab-case filename conventions.
- `CLAUDE.md` documents the collection and its defining rule: ideas are
  never load-bearing. No typed dependency edge may target an idea;
  untyped mentions of ideas, and provenance edges from a decision that
  adopts one, are permitted. Rejecting or abandoning an idea must
  invalidate nothing canonical.
- Idea lifecycle uses terminal states rather than deletion (for example
  `parked` to `adopted` or `rejected`), with placement and metadata in
  agreement, mirroring existing conventions.
- The parked ideas in log 0002 are migrated to individual idea artifacts,
  each citing its motivating evidence (dragons, decisions, prior art)
  using the interim prose reference convention from decision 0006.
- Log 0002 gains an appended note that its parking-lot role is retired;
  its existing content is not rewritten.

## Notes

References remain prose-form until dragon 3
(`drg_01KY169X7W0YXJ5QFV4D1MK4FB`, reference syntax) resolves; this task
must not block on it.

## Result

Established `archaeology/ideas/` with lifecycle directories `parked/`,
`adopted/`, `rejected/`, where status equals the directory name. Only
`parked/` was materialized: pre-creating empty terminal directories would
reproduce the Git round-trip flaw recorded as dragon 2, so the CLAUDE.md
conventions now state that lifecycle directories are created on first use.
That generalization also tightened the existing placement rule from the
`closed/`-specific wording to "status equals the lifecycle directory".

The five parked ideas in log 0002 were migrated to ideas 1–5
(`links bind` with check mode; doctor reference-graph checks;
`strata edit`; editor integration shims; typed dragon-resolution edges),
each with Problem / Sketch / Evidence sections citing dragons, decisions,
and prior art in the interim prose convention. Log 0002 gained an appended
retirement note; its rejected-alternatives record is unchanged.

CLAUDE.md documents the collection, the never-load-bearing rule, and the
`parked → adopted | rejected` lifecycle with terminal moves instead of
deletion.

## Verification

`scripts/check.sh` clean (fmt, 116 tests, clippy — no code changed).
`strata list dragons` and `strata show` remain unaffected by the new
directory, confirming reads only touch managed dragon paths. Front matter
of all five ideas hand-checked for sequence/filename/status/placement
agreement, since no tooling validates the `idea` kind yet.
