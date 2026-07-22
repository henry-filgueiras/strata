---
id: tsk-manage-ideas-collection
sequence: 14
kind: task
status: closed
sprint: spr-references-and-ideas
created: 2026-07-22
closed: 2026-07-22
---

# Manage the ideas collection

## Objective

Make `idea` the second CLI-managed collection by deliberate duplication
of the dragon mechanics (idea 10's rule-of-three discipline): create,
discover, list, show, transition, and validate ideas through Strata,
against the sixteen hand-seeded ideas already in this repository.

## Acceptance criteria

- `strata new idea "Title"` creates a parked idea with safe sequence
  allocation across all lifecycle directories, deterministic slug,
  generated stable identity with a distinct prefix, and refusal to
  overwrite — mirroring dragon creation semantics.
- `strata list ideas` and `strata list ideas --json` discover ideas
  across `parked/`, `adopted/`, and `rejected/`; `strata show idea:N`
  resolves across the same set. JSON output is deterministic.
- `strata adopt idea:N` and `strata reject idea:N` move a parked idea
  to its terminal directory and rewrite exactly the front-matter
  `status`, under the decision 8 (`dec-mutation-failure-classes`)
  contract, reusing the transition machinery from task 7. Transitions
  from a terminal state are refused; status/placement mismatches are
  refused with direction to `doctor`.
- Hand-seeded `idea-*` identities are read, listed, shown, and
  transitioned without rewrite, per decision 2's grandfathering update.
- `doctor` validates ideas with the same checks dragons get: placement
  and status agreement, malformed front matter, duplicate identities,
  duplicate sequences — and stays green on this repository.
- Tests mirror the dragon coverage in temporary directories: creation,
  collision refusal, discovery, resolution, both transitions,
  refusals, JSON determinism, malformed metadata.
- The task result records the duplication honestly: what was copied,
  what diverged and why, and what the copy taught about the eventual
  spec shape — this is the primary evidence input for idea 10.

## Notes

Lifecycle is `parked → adopted | rejected` only; no reopen analog is
implemented (an un-parking need has no recorded instance). Terminal
directories are created on first use per the dragon 2 convention —
`rejected/` does not exist yet and must not be pre-created.

## Result

All commands landed: `strata new idea`, `list ideas` (human and
`--json`), `show idea:N` and show-by-id across both collections,
`adopt`, and `reject`, with generated `ide_`-prefixed ULID identities
and the Problem/Sketch/Evidence template. Transition verbs are
collection-scoped: `close idea:1` and `adopt dragon:1` are refused
with the verbs that do apply, and a transition the lifecycle does not
define (rejecting an adopted idea) is refused naming the legal
lifecycle. `doctor` validates both collections, treats display
sequences as collection-scoped (`dragon:1` and `idea:1` coexist), and
enforces stable-id uniqueness globally.

The duplication verdict, as evidence for idea 10
([[idea-declarative-collection-specs|declarative collection specs]]):
almost nothing wanted to be duplicated. The byte-level machinery
(front-matter parse, safe write, status rewrite, the two-step
transition of decision 8) was already collection-agnostic, and the
collection-specific residue compressed into one plain-data descriptor
— `read::Collection { kind, states: [(status, dir)], transitions }` —
plus per-collection creation templates and command vocabulary, which
stay hardcoded. That descriptor is deliberately *not* the idea 10 spec
engine: no self-validation, no user-defined collections, no payload
codecs. But its shape is the strongest evidence yet for what the spec
wants to be: the second collection needed exactly kind, lifecycle
map, and transition table, nothing else. What data could NOT express:
the status vocabulary became one shared enum spanning both
collections (a closed set the spec engine would need to open), and
the transition *verbs* (`close`/`adopt`) remained command-surface
decisions data cannot make.

Divergences from the dragon story, recorded deliberately:

- `init` does not pre-create idea directories; all three materialize
  on first use (`new` creates `parked/`, the first `reject` creates
  `rejected/`), which is the dragon 2 convention applied more
  consistently than the dragon collection itself applies it.
- Ideas have no reopen analog; terminal means terminal, so the
  transition table is asymmetric where the dragon one is a cycle.
- `list` widened its status column from 6 to 8 characters for
  `rejected`; needle-based tests absorbed this without change.

Rust note for the case study: the lifecycle data wanted to be
`static` (not `const`) so `&'static Collection` references work
naturally, and clippy's `collapsible_if` now rewrites nested
`if let` + condition into Rust 2024 let-chains — the first time this
codebase used one.

## Verification

`scripts/check.sh` clean (fmt, 191 tests, clippy). New unit tests
cover idea creation (template, parked placement, first-use directory
materialization, sequence independence from dragons), three-directory
scanning, the idea status vocabulary, lifecycle-data legality, and
undefined-transition refusal; the new `tests/ideas.rs` integration
suite pins creation, listing (human and exact JSON), show by
reference and by hand-seeded id, adopt/reject byte-preservation,
terminal permanence, verb scoping, mismatch refusal, and not-found.
Dogfooded live: `doctor` green over 20 artifacts (4 dragons, 16
hand-seeded ideas — including idea 6's extra `adopted:` key, tolerated
as unknown metadata), `list ideas` renders all sixteen, `show idea:6`
returns the adopted specimen byte-for-byte.
