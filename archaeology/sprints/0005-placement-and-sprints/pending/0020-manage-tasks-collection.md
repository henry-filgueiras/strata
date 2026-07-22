---
id: tsk-manage-tasks-collection
sequence: 20
kind: task
status: pending
sprint: spr-placement-and-sprints
created: 2026-07-22
---

# Manage the tasks collection end to end

## Objective

Make `task` the fourth CLI-managed collection: tasks are created under
the active sprint's containment directory, discovered across all
sprints, listed, shown, and closed — and the duplication verdict from
building the third and fourth collections is recorded as the closing
evidence for idea 10's rule-of-three bet.

## Acceptance criteria

- `strata new task "Some work item"` requires an active sprint,
  allocates the next global task sequence, generates a `tsk_`-prefixed
  ULID, stamps the owning sprint's stable ID in the `sprint:` field,
  and creates the file in the active sprint's directory from a
  template carrying Objective and Acceptance criteria sections.
- `strata list tasks` discovers tasks across all sprint directories
  and renders sequence, status, sprint, age, and title; `--json` is
  deterministic; a filter for the active sprint's tasks exists in
  whatever form the interaction surface makes natural.
- `strata show task:17` and show-by-stable-id render the task.
- `strata close task:N` transitions `pending` to `closed` in place,
  stamping `closed:`, under the decision 8 failure classes.
- Hand-seeded `tsk-*` identities on tasks 1 through 21 remain valid
  and are never rewritten; `doctor` validates tasks, including that a
  task's `sprint:` field names an existing sprint and that its
  placement matches that sprint's containment directory.
- The duplication verdict — what the third and fourth collections
  shared, what stayed irreducibly per-collection, and what this proves
  or disproves about the descriptor shape — is recorded in this task's
  result as evidence for
  [[idea-declarative-collection-specs|idea 10]].
- `doctor` is green on this repository afterward.
