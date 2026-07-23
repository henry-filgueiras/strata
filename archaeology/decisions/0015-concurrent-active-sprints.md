---
id: dec-concurrent-active-sprints
sequence: 15
kind: decision
status: accepted
created: 2026-07-22
---

# Concurrent active sprints are legal

## Context

Thread 7 claim A ([[cmt-s5-placement-and-cardinality|placement and
sprint cardinality]], owed by
[[tsk_01KY64ZPXED0D4RGN8E219AXFB|task 28]]) established that the
single-active-sprint rule entered as one acceptance-criteria line in
task 19 with no decision record, contradicting the recorded owner
position in [[idea-cross-sprint-dependency-validity|idea 14]]:
concurrent active sprints are valid, and for disjoint work possibly
preferred; the interesting risk is cross-sprint *coupling*, not
concurrency. Sprints 2 and 3 ran concurrently without conflict. The
doctor error's rationale ("a state only a branch merge can produce")
was circular — merge-only because `new sprint` refused. And in the
very state doctor convicted, bare `new task` would have picked an
active sprint by scan order rather than refusing: the ambiguity
surface was never designed.

## Decision: cardinality

Henry ratified concurrent active sprints on 2026-07-22. Idea 14's
standing owner position therefore stands; it is cited here as
provenance and remains parked, unamended — this decision consumes its
cardinality position, while its dependency-validity sketch stays
future work.

- Active-sprint cardinality is not repository validity. Disjoint
  concurrent sprints are legal; `new sprint` no longer refuses while
  another sprint is active, and doctor's `multiple-active-sprints`
  error is retired outright. No successor finding exists at any tier:
  re-tiering the same cardinality claim as advice would keep alive an
  invariant no decision supports.
- Cross-sprint dependency validity and empty-frontier advice remain
  idea 14 / dragon 3 territory, untouched here.
- A review hold is procedural state, not an accidental use of sprint
  cardinality. Sprint 6's mechanical-interlock rationale described its
  opening state under the old rule; the incident hold runs through
  umbrella thread 3's explicit closure protocol. The orthogonal
  review-hold primitive remains parked as
  [[ide_01KY64ZPXVR0XRZBHKERBXXJ0C|idea 20]].

## Decision: task placement under concurrency

Explicit selection: `strata new task "<title>" --sprint <sprint-ref>`
accepts `sprint:N` or an addressable stable sprint id, resolves
exactly one sprint through the established typed resolution contracts
(not-found, ambiguity, malformed-corpus errors unchanged), refuses a
sequence reference into a non-sprint collection, and refuses a closed
sprint before writing. `--sprint` on dragon, idea, or sprint creation
is an invalid invocation. Explicit selection writes the task only
into the chosen sprint's containment directory; global task sequence
allocation is unchanged.

Bare `strata new task`:

- zero active sprints: the existing no-active-sprint refusal stands;
- exactly one active sprint: it is inferred, preserving the existing
  ergonomics;
- multiple active sprints: refused before any write, naming every
  active sprint deterministically and directing the user to
  `--sprint`. Scan order never resolves the ambiguity.

## Decision: the active projection

`list tasks --active` is the union of tasks owned by every active
sprint. Human and JSON projections use the same filtered set,
deterministic global task ordering is preserved, and tasks belonging
to closed sprints are excluded. No further selector or dependency
system rides along.

## Consequences

- Task 28 implements this decision: creation, explicit and inferred
  selection, the multi-active refusal, the union projection, and the
  retired doctor finding.
- The superseded `a_second_active_sprint_is_refused_naming_the_first`
  regression is replaced by decided concurrency behavior rather than
  left contradictory.
- When cross-sprint coupling becomes real, the deterministic
  dependency rule is designed under idea 14 with dragon 3's evidence,
  not retrofitted onto cardinality.
