---
id: ide_01KY7QF5FKX30PHTQ320MG4QXS
sequence: 21
kind: idea
status: parked
created: 2026-07-23
---

# Commission distributed agents into an inherited project

## Problem

Agents (and agent teams) inherit projects they did not build. The
industry default is *onboarding*: explain where things are and how
routine work is performed, then start working. Onboarding establishes
familiarity; it does not establish whether the agent has formed a
sufficiently accurate, evidence-backed operational model to perform
bounded work safely. The Sprint 5 post-merge review demonstrated the
gap: work landed on `main` from a confident operator whose review had
not completed, and repairing that required a full adversarial
protocol — reconnaissance, allegation, adjudication, owned repair,
verification — none of which any onboarding ritual provides.

*Commissioning* — the term is deliberately stronger than onboarding —
is a progressive grant of confidence and authority, not a one-time
context dump. Its output is durable project evidence that later humans
and agents can inspect, challenge, replay, and improve.

## Sketch

A provisional commissioning sequence, recovered from the executed
Sprint 5 post-merge review
([[cmt-sprint5-post-merge-stop-the-line|thread 3]] and
[[spr_01KY61D615FAC8VVSTD7QXX1DW|sprint 6]]):

1. Freeze and identify the baseline under examination.
2. Perform independent repository reconnaissance.
3. Open adversarial review threads for suspected contradictions,
   regressions, policy violations, or unowned risks.
4. Adjudicate allegations against executable evidence rather than
   accepting agent confidence as proof.
5. Route unresolved policy choices through explicit human gates.
6. Convert accepted findings into an owned repair DAG.
7. Execute the repair campaign as a bounded commissioning sprint.
8. Allow parallel task branches only where ownership and dependency
   boundaries are explicit.
9. Verify each repair independently and then verify the integrated
   result.
10. Close the commissioning campaign with a durable report of what was
    kept, repaired, reverted, deferred, and learned.

This sequence is a hypothesis extracted from one unusually rich
incident, not a finalized universal workflow.

Multi-agent design pressures the incident surfaced:

- reconnaissance should permit independent interpretations before
  synthesis, so one agent's framing does not silently become shared
  ground truth;
- allegations, accepted findings, policy decisions, implementation
  tasks, and verification evidence are different artifact classes or
  roles and must not be collapsed into one conversational transcript;
- disagreement between agents is useful evidence and should remain
  attributable;
- task parallelism must follow the repair dependency graph, not merely
  available agent count (the incident's wave plan serialized shared
  code seams even between nominally independent tasks);
- an agent may be commissioned for one bounded capability without
  being trusted for unrelated mutations;
- commissioning state may therefore eventually be scoped by
  repository, baseline, role, capability, or campaign rather than
  represented as one global boolean;
- re-commissioning may be required after material architectural,
  policy, tooling, or agent-model changes;
- closure must preserve deferred seams and negative findings, not
  merely successful repairs (the incident closed with thread 8
  accepted-deferred and its seam parked, and with rejected claims'
  refutations preserved).

## Boundaries

Against [[ide_01KY64ZPXVR0XRZBHKERBXXJ0C|idea 20]]: idea 20 concerns
a durable review hold or interlock. Commissioning may use such a hold,
but it also covers reconnaissance, adjudication, staged authority,
repair planning, execution, and demonstrated competence. Neither idea
should absorb the other.

Architectural boundary: Strata is a durable control plane, project
memory, and evidence ledger. Strata is not an autonomous-agent
runtime, scheduler, message bus, model router, sandbox manager, branch
worker, or general distributed workflow engine. External agent systems
may consume and produce Strata artifacts through stable human-readable
and machine-readable interfaces; those adapters are not designed here.

## Open questions

Deliberately unsettled:

- What observable evidence constitutes successful commissioning?
- Is commissioning attached to an agent identity, model/version, role,
  capability, baseline commit, repository, or some combination?
- Which permissions should be progressive, and which must always
  remain explicit human gates?
- How does the system detect that prior commissioning evidence has
  become stale?
- How should independent agent findings be compared without forcing
  premature consensus?
- Which parts of the Sprint 5 incident were essential protocol and
  which were accidental ceremony?
- Can the minimum useful protocol work with existing ideas, comments,
  decisions, tasks, sprints, doctor checks, and Git branches before
  introducing any new artifact kind?

## Evidence

The Sprint 5 post-merge review (2026-07-22) executed every stage of
the sketch sequence once, end to end: baseline frozen as
`bb3c18d..d98b363`; adversarial threads 4–9 opened against it;
every allegation independently verified, refuted, or narrowed against
code and history; the one genuine policy choice (sprint cardinality)
routed through an explicit human gate (decision 15); accepted findings
converted into the owned repair DAG of tasks 22–31 with recorded wave
ordering; each repair verified by re-run reproduction and the
integrated result audited file by file; and the campaign closed with a
durable kept/repaired/reverted/learned disposition. The term
"commissioning", and its contrast with onboarding, were named during
the post-incident consolidation (2026-07-23); the repository record
supplies the executed protocol, not the name.

## Promotion trigger

Before any product requirements, CLI commands, schemas, or a dedicated
commissioning artifact kind:

- run at least one additional bounded commissioning experiment on a
  substantially different inherited repository or subsystem;
- compare its stages and failure modes with the Sprint 5 incident;
- identify the smallest reusable protocol that survives both cases.

Candidate experiment (a design for the trigger, not an implementation
commitment): select a modest unfamiliar repository or isolated
subsystem; pin a baseline commit; give two agents independent
reconnaissance assignments; preserve their findings separately;
adjudicate a small sample through human policy gates; build and
execute a bounded repair DAG; measure false allegations, missed
defects, duplicated work, human interventions, verification failures,
and evidence needed for closure; record which existing Strata
primitives were sufficient and where coordination depended on
chat-only state or manual convention.

## Excluded for now

- autonomous permission escalation;
- unsupervised merging or deployment;
- generalized agent orchestration;
- reputation scores or universal agent trust rankings;
- treating a passing test suite as proof of architectural
  understanding;
- introducing a new lifecycle or artifact collection before another
  commissioning experiment supplies evidence.
