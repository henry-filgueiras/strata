---
id: spr-lifecycle-and-recall
sequence: 2
kind: sprint
status: closed
created: 2026-07-21
closed: 2026-07-22
---

# Sprint 2: Lifecycle and recall

## Goal

Make Strata own the transitions it currently watches others perform by
hand, and prove the read side earns ambient attention: close and reopen
dragons through the tool, and surface a forgotten risk with
`strata fortune`.

## Rationale

Sprint 1's retrospective counts three hand-performed lifecycle
transitions — each a move-plus-status edit that `doctor` can diagnose
but no command owns. Transitions are the most-demanded missing intent
operation and complete the dragon collection's story: create, discover,
validate, transition. Fortune (idea 6, `idea-strata-fortune`) is the
cheapest demonstration that recorded risks resurface without being
asked for — the read-rate answer to "is this a memory or a diary?".

Both tasks stay inside the hardcoded dragon collection, deliberately:
the second CLI-managed collection is the forcing function for
generalization (idea 10, `idea-declarative-collection-specs`) and
belongs to a later sprint with that design in hand.

## Success criteria

A user can run:

```sh
strata close dragon:1
strata reopen dragon:1
strata fortune
```

The implementation must:

- move an artifact between lifecycle directories and rewrite exactly its
  front-matter `status`, preserving every other byte;
- never lose content, per the failure-class contract of decision 8
  (`dec-mutation-failure-classes`): after any returned error and under
  abrupt process termination the artifact exists at exactly one path
  with valid contents; power-loss durability stays out of scope
  (dragon 4, `drg_01KY3C0S3JQKEMEB9BH6NVJ35F`);
- refuse transitions on artifacts whose status and placement already
  disagree, directing the user to `doctor`;
- keep `doctor` green across every successful transition;
- print one open dragon's reference, title, age, and excerpt on
  `strata fortune`, favoring stale artifacts, with a friendly empty-state
  message;
- adopt idea 6 (move it to `adopted/`) when fortune lands.

## Non-goals

This sprint does not implement:

- collections beyond `dragon` or the spec engine (idea 10);
- reference syntax or typed edges (dragon 3);
- `--commit` / single-invocation commits (idea 9; adoption needs a
  decision against the recorded non-goal);
- editor integration for transition prose (ideas 3 and 4);
- chores, staleness metadata, or ledgers (idea 7);
- fortune drawing from parked ideas — that waits for ideas to become a
  managed collection.

## Amendments

- 2026-07-21: the transition safety criterion is scoped by failure
  class. External review (thread `cmt-transition-crash-contract`, the
  idea 11 specimen) showed the original wording promised one atomicity
  level across all failures, which no portable filesystem primitive can
  deliver for an operation that changes both path and contents. The
  refined contract is decision 8 (`dec-mutation-failure-classes`); the
  residual power-loss exposure is dragon 4.

## Retrospective (2026-07-22)

Both tasks closed; every success criterion holds. `strata close`,
`strata reopen`, and `strata fortune` were dogfooded live in this
repository: a close/reopen round trip on dragon 1 left the archaeology
byte-identical with `doctor` green on both sides, and fortune surfaced
a real dragon with reference, title, age, and excerpt. Idea 6 is
adopted — the first artifact to complete the `parked → adopted`
lifecycle. The error contract grew one category
(`transition-interrupted`, exit 10) for the doubly-degraded rollback
decision 8 anticipated.

Durable learnings, recorded where they belong:

- the failure-class contract was implementable exactly as written; the
  only state that can leak is the one decision 8 named, and reaching
  its test coverage required an injectable seam around the two
  mutating primitives — external faults cannot time the gap between
  the transition's two steps (task 7 result);
- the strict read pipeline pays compound dividends: mismatch refusal,
  re-run refusal after interruption, and resolution semantics all fell
  out of reusing scan-plus-resolve rather than writing
  transition-specific validation (task 7 result);
- pinning randomized behavior structurally — a pure weight function
  plus a parameterized draw — kept fortune's tests deterministic while
  the amendment's exclusions (`--seed`, `--json`) cost nothing
  (task 8 result).

Friction to fix next: sprint and task closure remains hand-performed
archaeology (this closure edited four files by hand); sprints, tasks,
and ideas are still not managed collections, which is the forcing
function idea 10 already records. Concurrently, sprint 3
(`spr-community-standards`) remains active awaiting its human-only
settings task — the executor gap parked as idea 15.
