---
id: spr-lifecycle-and-recall
sequence: 2
kind: sprint
status: active
created: 2026-07-21
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
- never lose content: at every failure point the artifact exists at
  exactly one path with valid contents;
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
