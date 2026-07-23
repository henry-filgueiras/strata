---
id: tsk_01KY62E9VMB6HDNJWD31YS1FBP
sequence: 23
kind: task
status: closed
sprint: spr_01KY61D615FAC8VVSTD7QXX1DW
created: 2026-07-22
closed: 2026-07-22
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

One typed catalog now fronts every identity consumer. `edges::Catalog`
is built from the single `edges::harvest` pass; each admitted claim is
retained as an `edges::Claimant` — decoded string id and kind,
repository-relative path, optional sequence and title where
recoverable — carrying an explicit `edges::Disposition`: `Canonical`
(passes the canonical artifact parse of its managed position),
`Unassessed` (probe-only: the file sits at no strongly managed
position, so no canonical verdict exists), or `Rejected { class }`
(the canonical parse refused the file; the class is the stable doctor
problem class, e.g. `malformed-artifact`). Dispositions come from
re-running the exact `read` parsers on the claimant's managed
position, never from inferring validity off absent optional fields;
the regression pins a rejected claimant with fully recoverable
`sequence` and `title`.

Admission threshold, implemented exactly as decision 12 records it:
bounded read produced UTF-8, front-matter framing parses, the front
matter is YAML-parseable, and both `id` and `kind` are strings. An
`id` without a string `kind` is not a claim; malformed bytes, framing,
or YAML fabricate nothing (`claim_admission_threshold_is_exact`).
Admitted claims are retained through canonical rejection,
unmanagement, and collision. Collision semantics operate on the
decoded value: `id: x` and `id: "x"` collide
(`quoted_and_unquoted_id_spellings_claim_the_same_decoded_identity`).

Resolution is a three-way algebra (`edges::Resolution`): `Missing`,
`Unique`, or `Ambiguous` with every claimant. Catalog order is
explicitly path-sorted after the walk, independent of directory
enumeration or creation order; the determinism regression builds the
same relative path set in two temporary repositories under opposite
creation orders and asserts identical path-sorted claimant and
candidate lists (`catalog_order_is_path_sorted_under_opposite_creation_orders`).

First-wins consumers replaced — none remain in production:

- `edges::harvest_ids` (the `entry().or_insert()` first-seen map) is
  deleted, not wrapped; no compatibility helper survives.
- `transition::resolve_edge`'s stable-id arm (`.find(first match)`)
  now resolves through the catalog: `Missing` is `artifact-not-found`,
  `Ambiguous` is `ambiguous-reference` naming every claimant path in
  path-sorted order, refused before any mutation — the same contract
  the `kind:N` arm already honored, so the existing error text's
  advice to prefer stable ids is now sound. The `kind:N` arm reads the
  same catalog.
- doctor's typed-edge validation resolves each bound target through
  the catalog: a multiply claimed id is an error finding
  (`ambiguous-edge`, provisional name per decisions 4 and 12) naming
  the id and every claimant — never validated against, or convicted
  by, an arbitrary claimant (the thread 5 specimen 2b verdict flip is
  pinned dead).
- doctor's duplicate-identity check runs over the complete catalog:
  `duplicate-id` (subsuming the former managed-only map, one
  vocabulary per collision) fires for managed-plus-unmanaged,
  unmanaged-plus-unmanaged, and canonical-plus-rejected combinations,
  naming every claimant path with its disposition annotated.
  `duplicate-sequence` behavior is unchanged (managed,
  collection-scoped).
- doctor's misfiled-task check no longer selects the first sprint
  claiming an id: containment is judged against every claimant
  (any-match), with the duplicate itself separately reported.

Zero/one/many behavior: zero claimants remains not-found (binding)
and `dangling-edge` (doctor); exactly one claimant preserves the
current behavior including the deferred seam — a unique claimant that
is malformed or unmanaged may still serve as a provenance target when
a title is extractable
(`unique_rejected_claimant_binding_preserves_the_deferred_seam`),
recorded by decision 12 as preserved-not-blessed; multiple claimants
are ambiguous regardless of dispositions.

Evidence on the out-of-scope question the criteria asked to record:
the unique-malformed-target seam is exercised only through the bind
path; nothing observed here argues for or against admitting such
targets — the disposition machinery now makes a future refusal a
one-line policy change at `resolve_edge`.

Regression coverage (12 focused tests added; suite 173 unit + 61
integration assertions green): the five thread 5 specimen classes
(decision-and-task collision via the human/JSON agreement CLI test;
two unmanaged claimants; malformed duplicating healthy; stable-id
binding refused against ambiguity while `kind:N` stays refused;
doctor's verdict per state), plus the admission threshold, decoded-id
collision, disposition explicitness, deterministic ordering,
multiple-rejected-claimants, ambiguous-edge, pre-mutation refusal
with byte-identical source, and human/JSON classification agreement.

Boundaries preserved: no addressability enforcement (task 25 — no
character-class checks, marker-label parsing and status conformance
untouched), no degraded-creation behavior (task 27 — flat and
containment creation, strict scans, and success reporting unchanged),
no repository-valid bit, no payload retention (catalog entries stay
bounded metadata per thread 8), and task 22's bounds intact — the
catalog reads through `read_artifact_bytes` and the non-following
walk. `strata doctor` on this corpus: 60 artifacts, no problems — no
ambiguous ids in the real archaeology. `scripts/check.sh` green.
