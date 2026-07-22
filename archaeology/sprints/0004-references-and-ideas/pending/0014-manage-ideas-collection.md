---
id: tsk-manage-ideas-collection
sequence: 14
kind: task
status: pending
sprint: spr-references-and-ideas
created: 2026-07-22
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
