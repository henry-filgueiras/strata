---
id: tsk-strata-fortune
sequence: 8
kind: task
status: pending
sprint: spr-lifecycle-and-recall
created: 2026-07-21
---

# Ambient recall: `strata fortune`

## Objective

Implement `strata fortune` per idea 6 (`idea-strata-fortune`): print one
open dragon — reference, title, age, and a short excerpt — so recorded
risks resurface without anyone remembering to look.

## Acceptance criteria

- Output includes the human reference (`dragon:N`), title, age derived
  from the front-matter `created` date, and a few lines of excerpt.
- Selection favors staleness: older `created` dates are more likely,
  and every open dragon has nonzero probability. The exact weighting is
  an implementation detail; document it in the task result.
- Read-only: no repository mutation, no Git requirement, no new state.
- An empty or marker-only repository prints a friendly "no open risks"
  message and exits 0.
- Closed dragons are never selected.
- Tests pin the output shape and the empty state; selection tests
  assert membership in the open set rather than a specific pick. The
  staleness bias is pinned structurally, not statistically: weighting
  is a pure function from open-set metadata to weights, unit-tested
  directly (weight monotonic in age, nonzero for every open dragon),
  and the single random draw takes its RNG as a parameter.

## On completion

Adopt idea 6: move `archaeology/ideas/parked/0006-strata-fortune.md` to
`adopted/` with `status: adopted`, citing this task as the adopting
work. Note in the result any divergence from the idea's sketch (v1
draws only from open dragons; parked ideas join when ideas become a
managed collection).

## Amendments

- 2026-07-21: the tests criterion pins the staleness bias via a pure
  weight function with the RNG passed in, after external review asked
  whether selection should be reproducible and explainable — the
  original criteria ("older more likely, everywhere nonzero") were
  untestable from output samples alone. Deliberately excluded until a
  real automation consumer exists: `--seed`, `--json`, and any
  selection-metadata or method-identifier surface; the weighting
  remains an implementation detail, not a compatibility contract. If
  automation later needs deterministic "most neglected" retrieval,
  that is an ordering on `list` (or a recall sibling), not a seed on
  fortune. Provenance: thread `cmt-fortune-reproducibility`.
