---
id: idea-strict-doctor
sequence: 13
kind: idea
status: parked
created: 2026-07-21
---

# Strict-mode doctor: the layout is closed under supported containers

## Problem

`doctor` validates only the trees it manages, so its silence is
ambiguous: a green report means "the dragon collection is healthy", not
"the repository contains only what Strata understands". Two concrete
gaps, in increasing sharpness:

- an entire unmanaged tree is invisible — the comment-thread specimen
  landed in `archaeology/comments/` and doctor stayed green without
  ever seeing it;
- an unexpected sibling directory *inside a managed collection root* is
  invisible — artifacts in a typo'd or invented
  `archaeology/dragons/stale/` escape every malformed/duplicate check,
  which is silence-as-health at its most dangerous, since duplicate
  sequence and id detection silently loses its corpus.

## Sketch

Two checks of different strengths, preserving decision 5's asymmetry:
absence stays healthy (Git drops empty directories), unexpected
presence becomes a finding.

- **Default-mode kernel, independently shippable now:** within a
  claimed collection root, the lifecycle-directory set is closed. Any
  non-hidden entry under `archaeology/dragons/` other than `open/` and
  `closed/` is a finding. This needs no new configuration and closes
  the sharper gap above.
- **Strict mode (`doctor --strict`), opt-in:** the set difference
  between the directory tree under `archaeology/` and the declared
  supported containers must be empty — what exists is exactly what is
  supported. This depends on idea 10
  (`idea-declarative-collection-specs`): "supported" must be
  machine-declared, not hardcoded, for
  the check to mean anything. Today the difference is everything
  except dragons, so the flag would cry wolf until tool coverage
  grows; strict doctor is thereby the second forcing function for
  collection specs, after the second managed collection itself.

Strict findings are ordinary findings (exit 9): opting in means asking
for pedantry. No warning-severity taxonomy is introduced, so decision
4's exit-code contract is untouched.

A pleasant side effect once meaningful: the strict-mode finding count
*is* the tool-coverage gap — a measurable dogfooding-progress metric
for this repository.

## Evidence

Live instance: `archaeology/comments/` (thread
`cmt-transition-crash-contract`) is invisible to doctor today. Decision
5 (`dec-bootstrap-repo-marker`) settles the absence side of the
asymmetry; decision 4's update records the finding vocabulary as
provisional and revisitable, which a new finding code would exercise.
Idea 10 is the declaration dependency for the strict half; the
default-mode kernel has no dependency at all.
