---
id: tsk-manage-sprints-collection
sequence: 19
kind: task
status: closed
sprint: spr-placement-and-sprints
created: 2026-07-22
closed: 2026-07-22
---

# Manage the sprints collection end to end

## Objective

Make `sprint` the third CLI-managed collection: creation with safe
numbering and generated stable identity, discovery, listing, show, and
closure, with `doctor` coverage — on the placement model decided in
task 17 and landed in task 18.

## Acceptance criteria

- `strata new sprint "Some goal"` allocates the next sequence,
  generates a `spr_`-prefixed ULID, creates the sprint's containment
  directory and its `sprint.md` from a template carrying the
  Goal/Rationale/Success criteria/Non-goals sections, and refuses
  collisions per the decision 8 failure classes.
- `strata list sprints` renders sequence, status, age, and title;
  `--json` is deterministic and structured per decision 4.
- `strata show sprint:5` and show-by-stable-id render the sprint.
- `strata close sprint:N` transitions `active` to `closed` in place,
  stamping `closed:`; closing a sprint that still has pending tasks is
  refused with a message naming them; at most one sprint may be
  `active` at a time, and `new sprint` while one is active is refused.
- Hand-seeded `spr-*` identities on sprints 1 through 5 remain valid
  and are never rewritten; `doctor` validates the sprint collection
  with the same rigor as dragons and ideas.
- `doctor` is green on this repository afterward.

## Result

Sprints are the third CLI-managed collection. `strata new sprint`
allocates the sequence from existing containment directories,
generates a `spr_` ULID, creates `NNNN-slug/sprint.md` from the
Goal/Rationale/Success criteria/Non-goals template, and refuses while
any sprint is active, naming it and the way out. `list sprints`
(human and `--json`), `show sprint:N`, and show-by-stable-id all work
against the hand-seeded corpus — all five sprints listed and
validated with zero identity rewrites. `strata close sprint:N`
rewrites status in place, stamps `closed:` after the `created:` line
in the same atomic write, and is refused while pending tasks exist,
naming each one. `doctor` walks every containment directory
(structural findings for loose files, malformed `NNNN-slug` names,
and missing `sprint.md`), validates each sprint through the shared
parse pipeline, and enforces at most one active sprint
(`multiple-active-sprints`, error tier — a state only a branch merge
can produce, mirroring the duplicate-sequence stance).

Divergences that the flat-file machinery could not express, recorded
for idea 10: the artifact is a fixed-name file inside a per-artifact
containment directory; the display sequence rides the directory name,
not the filename (the parse core now takes the expected sequence and
its carrier from the caller); creation materializes a directory per
artifact; and closure consults a *different* collection (pending
tasks) before transitioning — the first cross-collection guard in the
tool. The `closed:` stamp became descriptor data (`stamp_closed`),
which dragons and ideas simply leave off.

Infrastructure for tasks (scanning, the `sprint:` field surfaced in
summaries and required on tasks) landed here because sprint closure
needs it; the task command surface is task 20.
