---
id: ide_01KY7S6GG3NAA35KBJTC6CA1TM
sequence: 23
kind: idea
status: parked
created: 2026-07-23
---

# Desire-path ledger: hand-performed operations as promotion evidence

## Problem

The strongest roadmap evidence Strata produces is the operation someone
had to perform by hand. The project already reasons this way, but only
retrospectively and only when a retrospective author happens to
remember: sprint 4's retrospective named sprint and task closure as the
last recurring hand-performed archaeology "for the third consecutive
sprint", which pitched sprint 5; sprint 6's retrospective named the
repeated manual front-matter-plus-`git mv` thread mechanics as
promotion evidence for [[idea-comment-threads|comment threads]]. When
the observation is not made at retrospective time, it is recoverable
only by re-reading diffs. Hand-performances that happen mid-task —
minting a decision by hand, appending a dated amendment section,
writing a log entry — leave no aggregated trace at all.

## Sketch

A convention before any machinery: when a session performs by hand an
operation Strata should conceptually own, it appends one dated row to a
single ledger artifact naming the operation shape (not the content) —
for example `2026-07-22 amend decision in place with dated section`.
Rows accumulate across sessions; at sprint-pitch time the ledger is
read and recurrence counts become promotion evidence, with the
project's working rule of three as the presumptive threshold for
sprint candidacy.

Relations. The [[idea-chore-artifacts|chore ledger]] shares the
one-row-per-performance grain but tracks recurring *maintenance*; this
ledger tracks *tool gaps*, and a row's ideal fate is that its
operation becomes a command and the rows stop. The CLAUDE.md
first-performance policy records the exact external command as dated
provenance inside the task that ran it; this ledger records only the
gap, aggregated in one place where recurrence is visible.

## Boundaries

- Not telemetry: rows are authored deliberately by the session that
  felt the gap; nothing writes the ledger automatically.
- The ledger is canonical prose or row-oriented data like any other
  artifact; projections may count it but must not replace it.
- Absence of a row proves nothing; the ledger only ever argues *for*
  promotion, never against it.
- Not a doctor concern: an out-of-date ledger is not corruption.

## Evidence

Sprint 4's retrospective (third consecutive sprint of hand-closure)
drove sprint 5's managed sprints and tasks. Sprint 6's retrospective
explicitly used "promotion evidence" for [[idea-comment-threads|ideas
11]] and [[ide_01KY64ZPXVR0XRZBHKERBXXJ0C|20]] — the pattern this idea
makes first-class was already performed twice, by hand, in
retrospectives. Decisions were minted by hand fifteen times before
sprint 7 proposed managing them; no ledger recorded that count — it
was reconstructed by listing a directory.

Proposed by Claude during the sprint 7 pitch, 2026-07-23.
