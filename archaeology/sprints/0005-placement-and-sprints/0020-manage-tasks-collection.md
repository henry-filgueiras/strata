---
id: tsk-manage-tasks-collection
sequence: 20
kind: task
status: closed
sprint: spr-placement-and-sprints
created: 2026-07-22
closed: 2026-07-22
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

## Result

Tasks are the fourth CLI-managed collection. `strata new task`
requires an active sprint (refused with guidance otherwise),
allocates the next global sequence across every sprint directory,
generates a `tsk_` ULID, stamps the owning sprint's stable id into
`sprint:`, and renders the Objective/Acceptance criteria template.
`list tasks` discovers across all sprints; `--active` narrows to the
active sprint (and is refused on other collections rather than
ignored); `--json` carries a `sprint` field that other kinds simply
omit. `show task:N`, show-by-id, and `close task:N` (in-place, with
the `closed:` stamp) work; all 21 hand-seeded tasks across five
sprints parse, list, and validate with zero rewrites. `doctor`
validates every task file and enforces the decision 11 cross-check as
`misfiled-task` (error tier): the `sprint:` field must name an
existing sprint whose containment directory holds the file.

The closing verdict on idea 10's rule-of-three bet, after four
collections: the spec-shaped residue is now `kind`, `dir`, states,
transitions, and one behavior flag (`stamp_closed`) — still plain
data. What refused to be data grew, though: sprints and tasks each
needed a bespoke scanner (fixed-filename-in-directory versus
files-across-directories), creation diverged at the sequence source
and destination, and two cross-collection rules appeared (sprint
closure consults tasks; task validity consults sprints) that no
per-collection descriptor can host. The evidence now says the idea 10
engine is two layers: a data spec for the common shape plus a small
set of behavior escape hatches for layout and cross-collection
guards — which is exactly the "behavior escapes to a trait only where
data cannot express it" seam the idea reserved.

One deliberate divergence from this task's acceptance criteria: the
human `list tasks` line renders reference, status, title, and path
like every other collection — the promised sprint and age columns
were dropped for output uniformity (the sprint is visible in the path
and the `--json` field; no list renders age today). If a sprint
column earns its keep, it should arrive as a deliberate list-format
decision for all collections, not a task-collection special case.

This Result was written while the task was pending; the transition
itself — `strata close task:20` — is the first task closure in this
repository performed by the tool.
