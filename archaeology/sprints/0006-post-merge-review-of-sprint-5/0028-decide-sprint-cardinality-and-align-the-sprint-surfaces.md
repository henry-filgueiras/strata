---
id: tsk_01KY64ZPXED0D4RGN8E219AXFB
sequence: 28
kind: task
status: closed
sprint: spr_01KY61D615FAC8VVSTD7QXX1DW
created: 2026-07-22
closed: 2026-07-22
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

## Result

Closed 2026-07-22. Claim A is remediated by
[[dec-concurrent-active-sprints|decision 15]]: Henry ratified
concurrent active sprints on 2026-07-22, so idea 14's standing owner
position stands. Idea 14 itself is cited as provenance and remains
parked, unamended; cross-sprint dependency validity and empty-frontier
advice stay its future territory (with dragon 3), and the review-hold
axis stays parked as idea 20.

**Creation.** `new sprint` no longer refuses while another sprint is
active; strict sprint scanning, sequence allocation, task 24 rollback,
and malformed-corpus refusal are unchanged. Multiple active sprints
are doctor-green: the `multiple-active-sprints` error is retired
completely with no successor at any tier — re-tiering the same
cardinality claim as advice would have kept alive an invariant no
decision supports.

**Selection.** `strata new task "<title>" --sprint <sprint-ref>`
accepts `sprint:N` or an addressable stable sprint id, resolves
through the established typed contracts, refuses non-sprint sequence
references and closed sprints before writing, and writes only into
the chosen sprint. `--sprint` on dragon/idea/sprint creation is
`invalid-invocation`. Bare `new task`: zero active sprints keeps the
existing refusal; exactly one infers it; multiple refuse before
writing, naming every active sprint deterministically (scan order
never resolves the ambiguity, closing the adjudication's corroborated
first-match gap). Global task sequences are unchanged and span
concurrent sprints.

**Listing.** `list tasks --active` is the union of tasks owned by
every active sprint; human and JSON share the filtered set,
deterministic global order is preserved, closed sprints' tasks are
excluded.

**Alignment.** Sprint 6's record carries a dated amendment: the
mechanical-interlock statement described its opening state, decision
15 supersedes the singleton assumption, and the incident hold runs
through umbrella thread 3's explicit closure protocol — Sprint 6
remains active. Code and CLI documentation stating "at most one
sprint may be active" is rewritten to the decided posture. The
superseded `a_second_active_sprint_is_refused_naming_the_first`
regression is replaced by decided concurrency behavior.

**Evidence.** Unit: `concurrent_active_sprints_are_created_normally`,
`concurrent_active_sprints_are_doctor_green`. Integration
(`tests/sprints.rs`, `tests/tasks.rs`):
`concurrent_active_sprints_are_created_normally_and_doctor_green`,
`bare_task_creation_with_multiple_active_sprints_refuses_naming_all`,
`explicit_sequence_selection_places_the_task_in_the_chosen_sprint`,
`explicit_stable_id_selection_places_the_task_in_the_chosen_sprint`,
`a_closed_selected_sprint_is_refused_before_writing`,
`non_sprint_selectors_and_misplaced_sprint_flags_are_refused`,
`task_sequences_are_global_across_concurrent_sprints`,
`list_tasks_active_is_the_union_across_all_active_sprints`; the
one-active inference and no-active refusal keep their existing tests,
and malformed containment still blocks sprint/task creation
(`sprint_and_task_creation_keep_the_strict_containment_boundary`).
Complete suite 345 tests green; `strata doctor` 60 artifacts, no
problems; `scripts/check.sh` passes; task 24 rollback and task 27
degraded-mode boundaries unchanged.
