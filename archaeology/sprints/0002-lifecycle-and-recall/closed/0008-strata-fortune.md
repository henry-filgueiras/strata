---
id: tsk-strata-fortune
sequence: 8
kind: task
status: closed
sprint: spr-lifecycle-and-recall
created: 2026-07-21
closed: 2026-07-22
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

## Result

`strata fortune` lands in `src/fortune.rs` as pure functions with the
rendering in `main`. Output is two header lines — `dragon:N  Title`,
then age and repository-relative path — followed by up to three lines
of body prose as the excerpt (headings, blanks, and fenced code
skipped). Implementation details, documented per the acceptance
criteria:

- The weight function is `max(age_days, 0) + 1`: linear in age from the
  front-matter `created` date, so monotonic, and at least 1 so every
  open dragon — including future-dated or unparseable stamps, which
  render as `opened today` / `age unknown` — stays reachable. Unit
  tests pin monotonicity and everywhere-nonzero directly, plus a
  reachability sweep over a full cumulative-weight cycle.
- The single draw is `pick(weights, roll)`, a cumulative walk taking
  its randomness as a parameter. The binary feeds it a fresh ULID's
  80-bit random component — real entropy already in the dependency
  tree, avoiding a `rand` dependency for one draw. Modulo bias over
  day-scale totals is negligible and the weighting is an
  implementation detail, not a contract.
- Read-only throughout: scan, filter to open, print. The empty and
  marker-only repository states print a friendly line and exit 0.

Idea 6 moved to `adopted/` with divergences recorded there: open
dragons only in v1, one weighted draw instead of two modes, and the
amendment's exclusions (`--seed`, `--json`) upheld.

## Verification

`scripts/check.sh` clean (fmt, 164 tests, clippy). Unit tests pin the
weight function's monotonicity and nonzero floor, the cumulative walk,
excerpt extraction, and age parsing/rendering; integration tests pin
the output shape, both empty states, degradation on an unparseable
`created`, no mutation across runs, and — over repeated invocations —
membership of every recall in the open set with closed dragons never
surfaced. Dogfooded live: fortune surfaced dragon 1 (reference, title,
`open 2 days`, path, excerpt) in this repository.
