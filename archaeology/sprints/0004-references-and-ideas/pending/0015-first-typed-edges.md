---
id: tsk-first-typed-edges
sequence: 15
kind: task
status: pending
sprint: spr-references-and-ideas
created: 2026-07-22
---

# Land the first typed edges and their doctor checks

## Objective

Put the vocabulary from task 13 to work: encode the first typed edges
in the corpus and teach `doctor` the per-kind semantics the decision
defined, so a dangling typed edge becomes a detected defect instead of
a silent one (idea 5's dangling-promise gap).

## Acceptance criteria

- Both closed dragons carry the decided resolution-provenance edge:
  dragon 2 to the artifact that settled it (decision 5's validity
  rule), dragon 3 to the task 13 decision.
- The one adoption specimen carries the decided adoption-provenance
  edge for idea 6 (`idea-strata-fortune`) and its adopting work, in
  whichever direction the decision fixed.
- `doctor` parses typed edges from front matter and enforces the
  decided semantics per kind — at minimum: an edge whose target
  identity does not exist in the repository is reported as the decision
  classifies it (corruption, not advice); an edge kind pointing at a
  kind the vocabulary forbids (any dependency edge targeting an idea)
  is likewise reported. Unknown edge kinds degrade as the decision
  specifies.
- Edge presence checks (should every closed dragon carry the edge?)
  follow whatever strength the decision assigned them; if the decision
  made presence advisory, `doctor` stays deterministic-only and the gap
  is noted for idea 13 (`idea-strict-doctor`).
- Tests cover: valid edges pass, dangling target fails, forbidden
  target kind fails, and the untouched corpus of this repository stays
  green after the retrofits.

## Notes

Retrofit scope is exactly the artifacts named above plus any file a
sprint 4 task already rewrites; bulk prose-reference migration is a
sprint non-goal.
