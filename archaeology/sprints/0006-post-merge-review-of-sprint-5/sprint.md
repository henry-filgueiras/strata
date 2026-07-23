---
id: spr_01KY61D615FAC8VVSTD7QXX1DW
sequence: 6
kind: sprint
status: closed
created: 2026-07-22
closed: 2026-07-22
---

# Sprint 6: Post-merge review of Sprint 5

## Goal

Adjudicate the late adversarial review findings against the merged
Sprint 5 range (`bb3c18d0..d98b3631`). Sprint 5 closed and landed on
`main` before its external review completed; the findings arrived
after the fact. This sprint receives each finding as an allegation,
independently verifies, refutes, or narrows it against the actual
code and history, and disposes of every thread with recorded
evidence. Accepted findings mint remediation tasks; rejected ones are
resolved with the refutation preserved.

## Rationale

This is a blameless stop-the-line incident, not a feature sprint. The
review gate was skipped by accident, not by policy change, and the
cheapest time to adjudicate findings is before new work stacks
assumptions on the disputed baseline. The single-active-sprint
invariant makes this sprint a mechanical interlock: while it is
active, no feature sprint can open, which is exactly the hold the
incident calls for. Sprint 5's implementation remains presumed
reviewable, not presumed broken — the burden of proof sits with each
finding, and no production-code change lands during adjudication.

The incident is also deliberate evidence: the review-thread specimen
it introduces (comment thread 3) is provisional test data for
ideas 11 and 19, not a settled schema.

## Success criteria

- Every blocking child thread of comment thread 3 is either repaired
  and verified, rejected with evidence, explicitly waived by the
  owner with a recorded reason, or converted into a narrowly scoped
  revert plus an archaeology-only salvage.
- Each adjudication records its evidence before any remediation task
  is minted; technical disposition and software disposition stay
  separate — accepting a defect does not itself revert Sprint 5.
- The incident's final disposition names what was kept, repaired,
  reverted, and learned.
- `scripts/check.sh` and `strata doctor` are green at close.

## Non-goals

- The spec-engine extraction (idea 10): its rule-of-three evidence is
  complete, but it waits behind this hold.
- Managed decisions as a fifth collection: named as a sprint 6
  candidate in Sprint 5's retrospective, deferred with the rest of
  the feature queue.
- Any unrelated Sprint 6 product work: this sprint's only output is
  adjudication, remediation tasks it mints, and archaeology.
- Treating reviewer allegations as accepted project truth: every
  claim starts as an allegation and earns its disposition through
  independent verification.

## Amendment: the hold does not rest on sprint cardinality (2026-07-22)

The Rationale above describes this sprint's opening state: when it
was written, the single-active-sprint invariant existed and made this
sprint an accidental mechanical interlock.
[[dec-concurrent-active-sprints|Decision 15]] (task 28) has since
superseded the singleton assumption — concurrent active sprints are
legal, and active-sprint cardinality is not repository validity.

The incident hold therefore does not, and never properly did, rest on
sprint cardinality: it remains authoritative through umbrella comment
thread 3's explicit closure protocol, which stays open and blocking
until its own criteria are met. Sprint 6 remains active. The
orthogonal review-hold primitive the original rationale gestured at is
parked as [[ide_01KY64ZPXVR0XRZBHKERBXXJ0C|idea 20]].

## Retrospective (2026-07-22)

This sprint closed ten tasks, 22–31. Every reviewer allegation was
independently adjudicated — verified, refuted, or narrowed against the
actual code and history — before any repair was minted, and no
production-code fix landed during adjudication. Sprint 5 was kept
rather than reflexively reverted: the keep-and-repair verdict held
through closure, with most accepted defects predating Sprint 5 and its
principal value (the deleted two-step transition failure class, stable
placement, managed sprints and tasks, transition-carried provenance)
retained in full.

All accepted findings received exactly one owner and executable
verification: threads 4, 5, 6, 7, and 9 resolved on re-run
reproduction evidence through tasks 22–30, thread 8 stayed
accepted-deferred with its seam parked in idea 18, and umbrella
thread 3 resolved "keep and repair; repaired and verified" after its
eight closure conditions were checked one by one. Task 31 records the
late ownership-boundary correction: decision 14's root-wide
attributes/config reach was an over-broad remediation minted inside
this incident, narrowed to archaeology-only on Henry's ratification —
the incident's own output received the same adversarial correction as
the work it reviewed.

Durable evidence deliberately left parked rather than promoted: the
hold ran on procedural authority once decision 15 removed the
accidental single-sprint mutex, which is idea 20's review-hold
evidence; and every thread resolution repeated the same manual
front-matter-plus-`git mv` lifecycle mechanics, which is promotion
evidence for idea 11's managed comment threads.

At close: doctor reports 61 artifacts with no problems, the complete
suite (348 tests) and `scripts/check.sh` pass, and no unrelated
product work entered the incident — the full `d98b3631..HEAD` diff
maps every change to an adjudicated owner.
