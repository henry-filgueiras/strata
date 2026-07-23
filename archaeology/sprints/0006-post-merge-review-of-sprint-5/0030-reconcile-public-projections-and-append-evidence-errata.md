---
id: tsk_01KY69B4ZXWQAJCWQYQCAZ7K3N
sequence: 30
kind: task
status: pending
sprint: spr_01KY61D615FAC8VVSTD7QXX1DW
created: 2026-07-22
---

# Reconcile public projections and append evidence errata

## Objective

Close the material findings of comment thread 9
([[cmt-s5-evidence-and-projection-drift|evidence and public projection
drift]]): README still asserts the retired lifecycle-directory model in
present tense and claims collections beyond dragons are maintained
manually, and two closed task Verifications record test counts that
were false at their own implementation commits. Repair the live
projection, append dated errata to the false historical assertions
without rewriting their original text, and record the one divergence
task 18's closure left unrecorded. This task owns only
evidence/projection repair — no implementation, representation,
identity, placement-policy, or comment-management work.

## Acceptance criteria

- README's "Artifact lifecycles" section no longer asserts that
  artifacts move between lifecycle directories, that status and
  placement must agree, or that terminal states are moves; it describes
  the decision 11 model: in-place front-matter transitions, stable
  paths, no lifecycle directories. Wording that touches placement
  *scope* — containment directories, the task collection's nesting, the
  comment collection — follows decision 11 as amended by
  [[tsk_01KY64ZPXPRBGH5S99G5E99TZY|task 29]], which must land first;
  this task must not restate the universal claim task 29 narrows.
- README's managed-collection posture matches the shipped surface: the
  claim that non-dragon collections "are maintained manually until
  then" is removed or re-scoped to the collections actually unmanaged
  (decisions, logs); the scoreboard and fortune description do not
  contradict the CLI (`new`/`list`/`show` over dragons, ideas, sprints,
  and tasks; `close`/`reopen`/`adopt`/`reject`; fortune drawing from
  open dragons *and* parked ideas). Showing a subset of commands
  remains acceptable; stating a false posture does not.
- Task 15's Verification gains a dated erratum: the recorded
  "213 tests" was false at implementation commit `c993e16`, where the
  suite was 200 (sum of all `cargo test` harness results, matching the
  `#[test]` inventory). The original text remains visible.
- Task 14's Verification gains the same dated erratum for its
  "191 tests": the suite at `e18c8e8` was 187. Task 16's "203" was
  verified exact at `3e06504` and needs no correction; the erratum may
  cite it as the convention control.
- Task 18 gains a dated note recording the divergence its closure left
  unrecorded: the acceptance criterion "no code path or document still
  references [the retired lifecycle directories] except historical
  records" was unmet at close — README's "Artifact lifecycles" section
  retained the retired model — while the Result's narrower claim
  (sample output and layout tree updated) was accurate. The original
  Result is not rewritten.
- Intentionally historical descriptions are untouched: closed sprint
  and task records, decision Context sections, and
  `bootstrap-inception.sh` (which reproduces the historical seed state
  and says so).
- Comment thread 9 is resolved under the manual comment-thread
  convention when this task closes.
- `strata doctor` and `scripts/check.sh` are green after the repair.
