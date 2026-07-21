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
- Mutation safety follows the failure-class contract of decision 8
  (`dec-mutation-failure-classes`). The transition is two atomic steps,
  status rewrite first: stage the full payload with the new `status`
  in a temporary beside the source and atomically replace the source,
  then atomically rename the source into the target lifecycle
  directory (no-clobber). Consequences the implementation must honor:
  - after any returned error the artifact exists at exactly one path
    with valid contents — unchanged where possible; if the rename
    fails after the status rewrite succeeded, the command rolls the
    status back, and only a doubly-failed rollback may leave the
    status/placement mismatch, which the error must then name;
  - under abrupt process termination the artifact exists at exactly
    one path with valid contents at every instant — never zero, never
    two; the only reachable defect is the status/placement mismatch
    between the two steps, which `doctor` reports precisely and which
    a re-run refuses per the mismatch bullet below;
  - power loss and kernel crashes are out of scope (dragon 4,
    `drg_01KY3C0S3JQKEMEB9BH6NVJ35F`).
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
- Tests pin the failure-class contract: fault injection at every
  returned-error boundary asserting the exactly-one-valid-artifact
  postcondition, including the rollback path; the crash-window
  intermediate state constructed directly on disk with its `doctor`
  diagnosis asserted; and byte preservation of everything except the
  `status` value across a successful transition.
- Human output names the artifact, both states, and the new path.

## Open design points

Resolution prose (a `## Resolution` section on closed dragons) stays
manual this task; whether the command should scaffold or prompt for it
is deferred to ideas 3, 4, and 9. Record the outcome of any template
decision in the task result.

## Amendments

- 2026-07-21: the mutation-safety criterion was rescoped by failure
  class and the test criteria expanded, after external review showed
  the original wording promised crash-spanning atomicity that a
  path-plus-contents change cannot portably provide. Provenance:
  thread `cmt-transition-crash-contract`; contract: decision 8;
  residual risk: dragon 4.
