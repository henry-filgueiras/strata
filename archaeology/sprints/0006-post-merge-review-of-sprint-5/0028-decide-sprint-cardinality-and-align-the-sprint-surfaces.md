---
id: tsk_01KY64ZPXED0D4RGN8E219AXFB
sequence: 28
kind: task
status: pending
sprint: spr_01KY61D615FAC8VVSTD7QXX1DW
created: 2026-07-22
---

# Decide sprint cardinality and align the sprint surfaces

## Objective

Close claim A of comment thread 7
([[cmt-s5-placement-and-cardinality|placement and sprint
cardinality]]). `new sprint` refuses while any sprint is active, and
doctor convicts `multiple-active-sprints` at error tier — a
cardinality invariant no decision record supports, contradicting the
owner position recorded in
[[idea-cross-sprint-dependency-validity|idea 14]] (concurrent disjoint
sprints valid, possibly preferred), which predates the sprint 5 pitch
and was never superseded. The rule entered as one acceptance-criteria
line in task 19 while sprint 5's non-goals explicitly deferred
idea 14's territory, and the doctor error's recorded rationale ("a
state only a branch merge can produce") is circular — the state is
merge-only because `new sprint` refuses.

Corroborating gap found in adjudication: `create_task` resolves "the"
active sprint by first match in scan order, so in the very state
doctor convicts, bare `new task` would pick a sprint arbitrarily
rather than refuse; no `--sprint` selector exists, and
`list tasks --active` assumes the singleton. The ambiguity surface was
never designed.

## Acceptance criteria

- A recorded decision adjudicates sprint cardinality against idea 14's
  recorded owner position. The default posture is that the recorded
  position stands; superseding it requires recorded reasons, and the
  current remediation sprint's accidental usefulness as a mutex is
  excluded as evidence (threads 3 and 7 — review holds are
  [[ide_01KY64ZPXVR0XRZBHKERBXXJ0C|idea 20]]'s axis).
- If concurrency is confirmed: `new task --sprint <ref>` selects
  explicitly; bare `new task` defaults only when exactly one sprint is
  active and refuses ambiguity rather than resolving it by scan order;
  `list tasks --active` semantics are decided for the multi-active
  case; the `new sprint` refusal is removed or narrowed per the
  decision; and doctor's `multiple-active-sprints` is retired or
  re-tiered with its successor semantics recorded. Idea 14's
  empty-frontier advisory remains future work gated on dragon 3 — this
  task decides cardinality, not dependency validity.
- If the singleton is instead ratified: the decision supersedes
  idea 14's position with reasons, idea 14 is amended to record the
  reversal, and the refusal and doctor error stand on decided ground.
- Either way, code documentation stating "at most one sprint may be
  active" matches the decision, and tests pin the decided selection
  and refusal behavior.
- `scripts/check.sh` and `strata doctor` are green at close.
