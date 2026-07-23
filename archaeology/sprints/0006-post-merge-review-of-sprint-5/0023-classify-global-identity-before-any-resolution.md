---
id: tsk_01KY62E9VMB6HDNJWD31YS1FBP
sequence: 23
kind: task
status: pending
sprint: spr_01KY61D615FAC8VVSTD7QXX1DW
created: 2026-07-22
---

# Classify global identity before any resolution

## Objective

Close the ambiguous-identity gap adjudicated in comment thread 5.
Stable ids are documented as globally unique, but that uniqueness is
only *checked* inside the strongly managed universe (dragons, ideas,
sprints, tasks), while typed-edge resolution trusts the best-effort
harvest of *every* front-matter id in the archaeology tree. Any id
collision with at least one claimant outside the strong set — a
decision and a task, two unmanaged artifacts, or a malformed-but-
harvestable file — draws no `duplicate-id` finding, and both doctor's
edge validation and the transition commands' stable-id provenance
binding silently resolve it to the traversal's first-seen claimant.
The `kind:N` binding path already diagnoses multiple matches with
`ambiguous-reference`; its error text even recommends the stable-id
path, which is precisely the path that chooses silently.

The invariant to implement, from the thread:

> Before resolution, one repository-wide header catalog classifies
> every candidate ID as missing, unique, or ambiguous. No command and
> no doctor check silently chooses among ambiguous identities.

## Acceptance criteria

- One repository-wide identity catalog, built from a single harvest
  pass, classifies every harvested id as unique or ambiguous and
  records every claimant's path and kind. Ids absent from the catalog
  are missing. Both doctor and the transition commands resolve through
  this classification; no caller consumes a first-seen-wins map.
- `doctor` reports every ambiguous id as an error finding naming all
  claimant paths, regardless of which claimants are strongly managed:
  managed-versus-unmanaged, unmanaged-versus-unmanaged, and
  malformed-but-harvestable claimants all surface. The existing
  managed-only `duplicate-id` behavior is preserved or subsumed —
  one collision never produces two competing findings vocabularies.
- Typed-edge validation never silently validates a bound edge against
  an arbitrary claimant of an ambiguous id: such an edge is an
  error-severity finding naming the id and every claimant.
- Stable-id provenance binding in the transition commands refuses an
  ambiguous id with the same `ambiguous-reference` contract the
  `kind:N` path already honors, naming every claimant path, so the
  existing error text's advice to use stable ids becomes sound.
- Regression tests cover the five specimen classes from thread 5:
  a decision and a task sharing an id; two unmanaged artifacts sharing
  an id; a malformed managed artifact whose harvestable id duplicates a
  healthy artifact's; stable-id binding against an ambiguous id
  (refused) versus `kind:N` binding (already refused); and doctor's
  verdict for each state.
- Whether a *unique* id whose only claimant is malformed or unmanaged
  may serve as a provenance target is out of scope here; if the
  implementation surfaces evidence either way, record it in the result
  rather than widening the change.
- `scripts/check.sh` and `strata doctor` are green at close; the real
  corpus must contain no ambiguous ids.

## Scope clarification (2026-07-22, comment thread 8)

The catalog retains every claimant, not every claimant's payload:
entries are bounded metadata (id, kind, sequence, title, path), the
shape `edges::Harvested` already demonstrates. Classifying an id as
ambiguous requires knowing all its claimants; it never requires
holding any claimant's body in memory
([[cmt-s5-read-cost-and-watermark|thread 8]]).

## Result
