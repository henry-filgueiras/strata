---
id: tsk-manage-sprints-collection
sequence: 19
kind: task
status: pending
sprint: spr-placement-and-sprints
created: 2026-07-22
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
