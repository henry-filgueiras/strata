---
id: tsk-reference-syntax-decision
sequence: 13
kind: task
status: closed
sprint: spr-references-and-ideas
created: 2026-07-22
closed: 2026-07-22
---

# Decide reference marker syntax and typed edge vocabulary

## Objective

Resolve dragon 3 (`drg_01KY169X7W0YXJ5QFV4D1MK4FB`): record the
decision that fixes the concrete inline marker syntax for untyped
references and the initial typed edge vocabulary with its front-matter
encoding, completing the two open points decision 6
(`dec-bootstrap-reference-model`) promoted to that dragon.

## Acceptance criteria

- A decision record fixes the inline untyped marker syntax in both
  strictness levels of one grammar: the canonical bound form (stable ID
  plus frozen label) and the unbound sugar form (sequence reference),
  per decision 6's one-grammar constraint.
- The decision shows both forms in a realistic prose sample and argues
  the raw-GitHub-diff readability test of decisions 6 and 7 against the
  rejected candidate, not just for the winner.
- The decision fixes the front-matter encoding for typed edges and an
  initial vocabulary containing only kinds with a consumer landing this
  sprint (dragon resolution provenance, idea adoption provenance).
  Each kind defines up front: direction, legal source and target kinds,
  the ideas-are-never-load-bearing exclusion where it applies, and its
  doctor semantics (what is corruption, what is diagnostic).
- The decision answers the sub-artifact fragment question from
  dragon 3 — admitting fragments into the target grammar or explicitly
  deferring them with the deferral's consequences stated.
- Decision 6 gains an update naming the successor; the interim prose
  convention is retired for new writing in CLAUDE.md's conventions.
- Dragon 3 is closed with `strata close dragon:3` (dogfooding
  sprint 2's deliverable), with the resolution recorded in the dragon
  per its own resolution criteria.

## Notes

The resolution provenance edge for dragon 3 itself is task 15's
retrofit, not this task's; closing here, encoding there, is the
expected order. Doctor enforcement of the decided semantics also lands
in task 15.

## Result

Decision 10 ([[dec-reference-syntax|reference syntax]]) chooses
wikilink markers over Markdown-link style. The decisive argument is
where each candidate fails the readability test: `[label](strata:id)`
fails at the *rendered* layer, because GitHub's sanitizer strips
non-allowlisted URI schemes and leaves bare label text — the reference
vanishes exactly where humans read most — while the wikilink renders
literally everywhere, noisy but never hiding the target. YAML quoting
was no tiebreak (both candidates begin with `[`, which YAML reads as a
flow sequence; edge values are therefore quoted strings).

Vocabulary landed smaller and more symmetric than dragon 3's candidate
list (supersedes, amends, resolved-by, implements, context-from):
only `resolved-by` and `adopted-by`, both instances of one rule —
terminal lifecycle states carry a provenance edge to the work that
terminated them — because only these have consumers this sprint.
The fragment question is answered by deferral with `#` reserved in the
target grammar, making the eventual extension non-breaking. Doctor
semantics per kind are three-tiered: corruption (unparseable value,
dangling bound target, forbidden target kind — the tier that
structurally keeps typed edges off ideas), repairable (sugar values,
lifecycle-contradicting edges), advisory (absence; promotion is
idea 13's question). The verification universe is every front-matter
`id` in the archaeology tree, so edges can target decisions and tasks
before those collections are managed.

Decision 6 carries the successor update; CLAUDE.md's conventions
retire the interim prose convention for new writing; dragon 3 carries
its resolution section and was closed with `strata close dragon:3` —
the third live dogfooding of sprint 2's transition machinery.

## Verification

`scripts/check.sh` clean (fmt, 164 tests, clippy — no code changed).
`strata close dragon:3` performed the transition; `strata doctor`
green (4 artifacts) after the move. The decision's own references use
the grammar it fixes, as do the updates to decision 6 and dragon 3 —
the first bound markers in the corpus.
