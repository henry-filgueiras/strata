---
id: tsk-first-typed-edges
sequence: 15
kind: task
status: closed
sprint: spr-references-and-ideas
created: 2026-07-22
closed: 2026-07-22
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

## Result

The corpus now carries three typed edges, all verified by `doctor`:
`resolved-by` on dragon 2 (to decision 5) and on dragon 3 (to
decision 10), and `adopted-by` on idea 6 (to task 8,
`tsk-strata-fortune`). The last one settled a small open point in the
decision's favor: adoption provenance lives on the idea pointing at
the adopting work — terminal states carry the edge — matching
`resolved-by` exactly.

Implementation is a new `src/edges.rs`: the marker parser (the
decision's grammar, `#` rejection included, reusable by the future
bind operation), the vocabulary as data (`EDGE_KINDS`: key, source
kind, settled status, legal target kinds — ideas absent from every
target list, so the never-load-bearing rule is enforced structurally),
best-effort id harvesting over the whole archaeology tree, and
per-artifact edge validation. `doctor` grew the decision's severity
tiers: findings now carry `severity: error | advice`, only errors make
the repository unhealthy (exit 9 counts errors alone), and advice
renders as prefixed lines plus an advisory count in the healthy
summary. New problem codes: `invalid-edge` and `dangling-edge`
(errors), `unbound-edge` and `stale-edge` (advice). The `--json`
findings contract gained the `severity` field — the one compatibility
surface this task changed, absorbed by the pinned-shape test.

The YAML footgun decision 10 predicted is real and now has a precise
diagnostic: an unquoted `[[...]]` value parses as a nested YAML list,
and the `invalid-edge` detail names the quoting fix.

## Verification

`scripts/check.sh` clean (fmt, 213 tests, clippy). Unit tests pin the
marker grammar (bound/sugar/eleven rejection cases), the
no-idea-targets vocabulary invariant, and every doctor tier: valid
edge to an unmanaged decision passes, dangling target errors, idea
target errors, wrong-source errors, unquoted-YAML errors, sugar and
lifecycle-contradiction advise without failing, unknown keys stay
inert. Integration tests pin the advisory exit-0 path (human line,
count, JSON severity) and the dangling-edge exit-9 path. Dogfooded
live: `doctor` green over 20 artifacts with all three retrofitted
edges resolving; a scratch-copy probe with a corrupted target id
produced `dangling-edge` and exit 9 against the real corpus.
