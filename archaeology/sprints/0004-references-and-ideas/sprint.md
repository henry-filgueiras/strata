---
id: spr-references-and-ideas
sequence: 4
kind: sprint
status: closed
created: 2026-07-22
closed: 2026-07-22
---

# Sprint 4: References and the ideas collection

## Goal

Resolve the reference-syntax dragon and make `idea` the second
CLI-managed collection: fix the inline marker grammar and the initial
typed edge vocabulary by decision, give ideas the same
create/discover/validate/transition story dragons have, land the first
typed edges alongside their first consumers, and let fortune draw from
the enlarged pool.

## Rationale

Every thread from the closed sprints' retrospectives converges here.
Sprint 2's "friction to fix next" names hand-performed archaeology and
unmanaged ideas, with idea 10 (`idea-declarative-collection-specs`) as
the recorded forcing function — and idea 10's own discipline prescribes
building the second concrete collection by deliberate duplication of
the dragon mechanics, extracting the spec engine only from working
duplicated code. Dragon 3 (`drg_01KY169X7W0YXJ5QFV4D1MK4FB`) names its
own deadline: resolve before or alongside its first consumer, the ideas
collection. Fortune drawing from parked ideas was a sprint 2 non-goal
gated exactly on ideas becoming managed. Idea 14
(`idea-cross-sprint-dependency-validity`) stays parked but is a
downstream consumer of the vocabulary this sprint fixes.

A second managed collection doubles the exposure surface of dragon 1
(`drg-bootstrap-branch-collisions`, sequence collisions across
branches); this is accepted unchanged — collision detection remains
`doctor`'s job and repair remains future work.

## Success criteria

A user can run:

```sh
strata new idea "Some proposal"
strata list ideas
strata list ideas --json
strata show idea:12
strata adopt idea:16
strata reject idea:16
strata fortune
```

The sprint must deliver:

- a decision record fixing the inline marker syntax (bound and unbound
  forms of one grammar) and the initial typed edge vocabulary — each
  kind introduced only with a consumer landing this sprint, each with
  its doctor semantics defined up front — answering the sub-artifact
  fragment question, passing the raw-diff readability test of
  decision 7, and retiring the interim prose convention for new
  writing; dragon 3 closed via `strata close dragon:3`;
- the `idea` collection managed end to end: creation with safe
  numbering and generated stable identity, discovery, listing (human
  and `--json`), show, `adopt`/`reject` transitions under the
  decision 8 failure-class contract, and full `doctor` coverage;
  hand-seeded `idea-*` identities remain valid and are never rewritten
  (decision 2 update);
- the first typed edges present in the corpus (dragon resolution
  provenance on both closed dragons, adoption provenance for the one
  adopted idea) with `doctor` verifying them per the decided semantics;
- `strata fortune` drawing from open dragons and parked ideas with the
  same staleness weighting;
- `doctor` green on this repository after every retrofit.

## Non-goals

- The spec engine itself (idea 10): extraction waits for a third
  collection; this sprint deliberately duplicates and records the
  duplication pain as evidence.
- Sprints and tasks as managed collections.
- `strata links bind` (idea 1): the decision defines the unbound sugar
  form; the repair command waits for a consumer.
- Reference-graph checks beyond typed-edge verification (idea 2's
  backlink/cycle projections).
- Bulk migration of historical prose references to the new marker
  syntax: new writing uses the decided grammar; retrofits happen only
  where a task in this sprint already touches an artifact.
- Comment threads (idea 11), frontier projection (idea 8), cross-sprint
  dependency enforcement (idea 14), relevance surfacing (idea 12).

## Retrospective (2026-07-22)

All four tasks closed in one day; every success criterion holds.
Decision 10 ([[dec-reference-syntax|reference syntax]]) fixed wikilink
markers and the terminal-provenance vocabulary, closing dragon 3
through the tool. Ideas are the second managed collection — the
sixteen hand-seeded specimens listed, shown, and transitioned without
a byte of identity rewriting. The corpus carries its first three
verified typed edges, `doctor` gained the decision's severity tiers,
and fortune now resurfaces parked ideas (dogfooded immediately:
consecutive draws surfaced five different ideas).

Durable learnings, recorded where they belong:

- The rule-of-three bet inverted: the second collection produced
  almost no duplication. The byte-machinery was already
  collection-agnostic, and the residue compressed into a plain-data
  descriptor (kind, lifecycle map, transition table) that is the
  strongest evidence yet for idea 10's spec shape — while creation
  templates and command verbs stayed irreducibly per-collection
  (task 14 result).
- Provenance direction generalized cleanly: terminal lifecycle states
  carry the edge to the work that terminated them, one rule
  instantiated twice (decision 10; task 15).
- The decision's severity tiers forced `doctor` to grow an advice
  channel; "legal but weak" now has an exit-0 rendering and a `--json`
  field instead of being unrepresentable (task 15 result).
- Wikilink over Markdown-link was decided by where each fails the
  no-tooling test: GitHub's scheme allowlist makes `[label](strata:id)`
  vanish at render time, while `[[...]]` is noisy but never hides the
  reference (decision 10; task 13 result).

Friction to fix next: sprint and task closure is *still*
hand-performed archaeology — this closure again edits the sprint file
and four task files by hand while `strata` watches, and sprints and
tasks remain the largest unmanaged collections. The collection
descriptor from task 14 lowers the cost of managing them, but tasks
nest under per-sprint directories, which the current
one-directory-per-state model cannot express; that layout question is
the real blocker and belongs to whatever sprint takes them on. The
`strata close --resolved-by` observation from decision 10 (letting
provenance ride the transition) is the natural companion piece.
