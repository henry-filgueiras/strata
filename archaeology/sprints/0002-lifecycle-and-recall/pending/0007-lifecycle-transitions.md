---
id: tsk-lifecycle-transitions
sequence: 7
kind: task
status: pending
sprint: spr-lifecycle-and-recall
created: 2026-07-21
---

# Close and reopen dragons through the tool

## Objective

Implement `strata close dragon:N` and `strata reopen dragon:N` (both
also accepting a stable id), so lifecycle transitions become intent
operations the tool owns instead of hand-performed move-plus-edit pairs.

## Acceptance criteria

- A successful transition moves the artifact to the target lifecycle
  directory and rewrites exactly the front-matter `status` value;
  every other byte of the payload is preserved.
- Mutation safety matches the existing write contract: staged content,
  no-clobber persist, and at every failure point the artifact exists at
  exactly one path with valid contents — never zero, never two.
- Typed refusals, reusing existing categories where they fit:
  - unknown or ambiguous references (`artifact-not-found`,
    `ambiguous-reference`);
  - transition to the state the artifact is already in;
  - a destination collision (same filename already in the target
    directory);
  - an artifact whose status and placement already disagree — the
    transition names the mismatch and directs the user to `doctor`
    rather than silently repairing.
- The destination directory is materialized on demand per decision 5.
- `doctor` reports a healthy repository after every successful
  transition, covered by tests.
- Human output names the artifact, both states, and the new path.

## Open design points

Resolution prose (a `## Resolution` section on closed dragons) stays
manual this task; whether the command should scaffold or prompt for it
is deferred to ideas 3, 4, and 9. Record the outcome of any template
decision in the task result.
