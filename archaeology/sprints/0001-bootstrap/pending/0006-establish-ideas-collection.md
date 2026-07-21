---
id: tsk-bootstrap-ideas-collection
sequence: 6
kind: task
status: pending
sprint: spr-bootstrap
created: 2026-07-20
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
