---
id: tsk_01KY69B4ZXWQAJCWQYQCAZ7K3N
sequence: 30
kind: task
status: closed
sprint: spr_01KY61D615FAC8VVSTD7QXX1DW
created: 2026-07-22
closed: 2026-07-22
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

## Result

Closed 2026-07-22. The live projection is repaired and every false
historical assertion carries a dated erratum beside its preserved
original text; no production code changed.

**README.** The "Artifact lifecycles" section now states the decision
11 model: transitions rewrite front matter in place, canonical paths
stay stable across lifecycle changes, lifecycle directories are not
part of managed collection semantics, and terminal states are
transitions, never deletions. The placement paragraph follows
decision 11 as amended by [[tsk_01KY64ZPXPRBGH5S99G5E99TZY|task 29]]
(landed first): stable containment is collection-specific — dragons
and ideas flat, sprints owning stable containment directories with
tasks inside — without restating the retired universal-flat claim.
The Status section no longer claims non-dragon collections are
maintained manually: dragons, ideas, sprints, and tasks are stated as
managed with `new`/`list`/`show` covering all four and
`close`/`reopen`/`adopt`/`reject` as the lifecycle commands, while
decisions and logs remain manual; the scoreboard keeps its dragon
spelling as an acceptable subset, and the fortune row now draws from
open dragons and parked ideas. No Task 26–28 release-note detail was
added.

**Errata.** Task 15's Verification gains a dated erratum: "213 tests"
was false at implementation commit `c993e16`, where the complete
suite was 200 under the sum-of-harnesses convention. Task 14's gains
the same: "191 tests" was false at `e18c8e8`, where the suite was
187. Both cite task 16's exact 203 at `3e06504` as the convention
control; task 16 itself is untouched. Task 18 gains a dated
divergence note: its no-retained-lifecycle-model criterion was unmet
at close because README's live section retained the retired model,
while its narrower Result (sample output, layout tree, inception
script) was accurate; the original Result and criterion are not
rewritten.

**Historical exclusions.** Closed sprint/task descriptions, decision
Context sections, `bootstrap-inception.sh`, task 20's honestly
recorded divergence, sprint 5's retrospective, and task 16 are
untouched; no opportunistic prose cleanup rode along.

**Verification.** Changed paths are exactly README.md, tasks 14, 15,
18, and this record, plus thread 9's resolution move. Complete suite
345 tests green; `strata doctor` 60 artifacts, no problems;
`scripts/check.sh` passes.
