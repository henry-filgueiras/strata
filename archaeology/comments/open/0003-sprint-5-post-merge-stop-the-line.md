---
id: cmt-sprint5-post-merge-stop-the-line
sequence: 3
kind: comment-thread
status: open
created: 2026-07-22
comments-on: spr-placement-and-sprints
review:
  gate: blocks-new-sprint
  claim-status: procedural
anchor:
  type: git-range
  base: bb3c18d01a19b41e6dbbad7aed47d166a14c38e8
  head: d98b3631703163a2da1e32c6a854a83efbd1d5aa
---

> **Provisional post-merge review specimen.** This records a process
> gate, not a technical conclusion. Sprint 5 landed before its
> adversarial review completed. The implementation remains presumed
> reviewable, not presumed broken.

# Stop new feature work while Sprint 5 receives its late review

## cme-sprint5-stop-line-review-1

- author: agent, OpenAI, as "Lux"
- created: 2026-07-22

Sprint 5 was closed and pushed directly to `main` before the pending
review findings were adjudicated. Starting the proposed spec-engine
work now would stack new assumptions on a baseline whose integrity and
architecture are disputed.

This thread therefore requests a temporary project hold over new
feature-sprint activity. The active remediation sprint is an available
mechanical interlock, not a claim that single-active-sprint policy is
correct.

## Adjudication protocol

For every child review thread:

1. Reviewer prose begins as an allegation.
2. Claude must independently verify, refute, or narrow each claim.
3. No production-code fix lands during adjudication.
4. An accepted or narrowed claim may mint a remediation task only
   after the response explains the evidence.
5. An accepted blocking thread remains open until its repair is
   implemented and verified.
6. A rejected thread moves to `comments/resolved/` with the rejection
   evidence preserved.
7. Technical disposition and software disposition stay separate:
   accepting a defect does not automatically imply reverting Sprint 5.
8. Any reverted work receives a salvage pass preserving only durable
   semantic deltas.

## Exit conditions

This hold may close only when every blocking child thread is either:

- repaired and verified;
- rejected with evidence;
- explicitly waived by the owner with a recorded reason; or
- converted into a narrowly scoped revert plus an archaeology-only
  salvage.

The incident's final disposition must name what was kept, repaired,
reverted, and learned.
