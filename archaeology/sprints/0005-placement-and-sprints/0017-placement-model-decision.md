---
id: tsk-placement-model-decision
sequence: 17
kind: task
status: closed
sprint: spr-placement-and-sprints
created: 2026-07-22
closed: 2026-07-22
---

# Decide one placement model for all collections

## Objective

Write the decision record that ends the repository's three-way
placement inconsistency: dragons and ideas file under lifecycle
subdirectories, decisions share one heterogeneous directory, and tasks
split into per-sprint `pending`/`closed`. The proposal on the table is
flat placement — one directory per collection, lifecycle state carried
only in front matter, per-sprint directories re-founded as pure
containment — with front-matter-driven recursive placement and the
status quo recorded as rejected alternatives.

## Acceptance criteria

- A decision record exists fixing the placement model for every
  current and future collection, covering: directory layout per
  collection; front matter as the sole authority for lifecycle state;
  what replaces `doctor`'s status/placement agreement check and which
  severity tier any successor check occupies; the fate of the
  decision 8 two-step transition contract when transitions no longer
  move files; and sequence/identity rules confirmed unaffected.
- The rejected alternatives are recorded with reasons: lifecycle
  subdirectories (double bookkeeping, moves break path stability),
  front-matter-driven recursive placement (migration and tooling cost
  for a projection the tool can compute), and the heterogeneous status
  quo (three patterns in code forever).
- The known cost — status filters scan the whole collection including
  the terminal long tail — is stated in the decision with
  [[ide_01KY5X7C56KBFWJJJKHTEXXQXV|idea 18]] cited as the parked
  counter-lever, not built.
- The decision passes the decision 7 raw-diff readability test.
- CLAUDE.md's manual archaeology layout and conventions sections are
  updated to the decided model, including retiring the
  status-equals-directory-name convention and the first-use directory
  convention where it no longer applies.
- No code changes in this task; migration and code alignment are
  task 18.

## Result

Decision 11 ([[dec-flat-placement|flat placement]]) is recorded and
accepted: all artifacts of a collection live directly in its
directory, front matter is the sole lifecycle authority, state changes
never move files, and per-sprint directories are re-founded as pure
containment. The decision retires the decision 8 two-step transition
contract (transitions become one safe write), retires `doctor`'s
status/placement agreement check in favor of two error-tier
successors (no strays outside collection directories; task `sprint:`
fields must match containment), confirms decision 2's identity and
sequence rules untouched, and records the accepted cost with idea 18
as the parked counter-lever. All three alternatives are recorded with
reasons, the strongest being the observed one: decisions have been
flat since sprint 1 with zero felt pain.

CLAUDE.md's layout tree and conventions are updated to match; the
status-equals-directory-name and lifecycle-directories-on-first-use
conventions are retired. This task file itself is closed under the old
convention — moved to `closed/` — as the last artifact to ever do so;
task 18's migration sweeps it flat along with everything else.

Raw-diff readability (decision 7): the decision is one Markdown file
of prose and one fenced tree diagram; nothing in it requires tooling
to render.
