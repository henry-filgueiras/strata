---
id: idea-capability-constrained-work
sequence: 15
kind: idea
status: parked
created: 2026-07-22
---

# Capability-constrained work and an up-for-grabs filter

## Problem

Task 12 (`tsk-github-community-settings`) is the framework's first
specimen of work its usual executor could not perform: flipping GitHub
settings requires admin web-UI access an agent session did not have.
Nothing in task metadata distinguishes it from any other pending task,
so an agent surveying pending work cannot tell which items are within
reach, and a human cannot ask the inverse — "what is waiting on
affordances only I have?" The gap generalizes as more executors with
different affordances (humans, agent harnesses with varying tool
access, CI) share one archaeology.

## Sketch

Tasks may declare **required affordances** in front matter (for
example `requires: [github-admin-ui]`); absence means unconstrained,
so tagging never becomes ceremony. Affordances describe the invoking
*session*, not the executor's species: human-versus-agent is the wrong
axis, since an agent with browser automation and delegated credentials
can flip repository settings while a human without push rights cannot
push. Task 12 is not "human-only"; it requires GitHub admin web-UI
access, whoever holds it.

The read side is a filter, not a scheduler: pending work intersected
with the invoker's declared capability set, composed over the frontier
[[idea-frontier-projection]] computes — "up for grabs" means unblocked
*and* within capability. The inverse filter serves the human
maintainer: work that waits on affordances only they hold.

Three boundaries keep it honest:

- **Advisory, never an ACL.** Strata cannot verify a capability claim,
  so it must not pretend to enforce one; the filter is advice, and
  performing work "outside" one's declared set is not a violation
  ("semantic systems advise; they do not define truth").
- **No claiming.** Up-for-grabs presents; choosing remains with the
  caller. Multi-agent locking is a recorded non-goal, and the
  presenting-versus-choosing gap idea 8 preserves is load-bearing here
  identically.
- **Vocabulary with first consumers.** Affordance terms are introduced
  only alongside the first task that needs them, mirroring dragon 3's
  edge-vocabulary rule; no speculative capability taxonomy.

## Evidence

Motivating instance: task 12 (2026-07-22), tracked as pending-for-a-
human only via prose in its objective; the executor gap was recorded in
sprint 3's rationale and promoted here after owner review the same day.
Prior art is strong and directly shaped: CI runner matching (GitHub
Actions `runs-on`, GitLab runner tags, Buildkite agent tags) has jobs
declare required labels and workers self-declare capabilities, with the
scheduler matching on containment — including the same trust model,
since a runner's labels are asserted, not verified. Issue-tracker
labels (`help wanted`, `good first issue`) are the advisory,
human-facing ancestor.
