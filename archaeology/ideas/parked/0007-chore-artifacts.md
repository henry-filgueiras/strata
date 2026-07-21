---
id: idea-chore-artifacts
sequence: 7
kind: idea
status: parked
created: 2026-07-21
---

# Chore artifacts: recurring maintenance with staleness and a ledger

## Problem

Repositories accumulate recurring obligations that are nobody's task:
keep the README status table in sync, refresh dependencies, re-verify
documented examples. The current taxonomy has no home for them — not
dragons (no unresolved question), not sprint tasks (never *done*), not
decisions (nothing settled). They live in heads and go stale silently.
Motivating instance: the 2026-07 README makeover left "update the status
table when `doctor` lands" with nowhere to live.

## Sketch

A canonical `chore` collection, load-bearing unlike ideas, lifecycle
`active` → `retired`. Front matter declares a staleness tolerance
(`stale_after: 90d` — a tolerance, not a cron cadence: nothing is *wrong*
at day 31 of 30, so whining ramps rather than fires). The payload pairs a
narrative definition with an append-only execution ledger: one row per
performance, recording actor, timestamp, optional effort measures, and
notes.

Ledger rows are JSONL, not a Markdown table: machine-appendable without
prose parsing, one-line diffs, trivial merges — and the first honest
exercise of the row-oriented payload promised by decision 3
(`dec-bootstrap-payload-separation`).

Nagging belongs to `strata fortune` ([[idea-strata-fortune]]): overdue
chores join open dragons in the ambient-recall pool. Staleness is
advisory; `doctor` checks only ledger well-formedness (malformed row =
corruption, overdue chore = advice), mirroring "semantic systems advise;
they do not define truth."

No stored commit pointers. A row cannot contain the hash of the commit
that contains the row, and it doesn't need to: the commit policy already
lands the ledger row in the same commit as the work it records, so
`git blame` on the row line *is* the provenance pointer — always correct,
recomputed after rebase, maintained by Git for free. A future Git-aware
`strata chore history` can join rows to blame. Outside Git, actor and
timestamp still function; only provenance is lost, and provenance is
Git-aware by design.

Effort metrics (wall-clock, tokens) are optional fields with an honesty
norm: recorded only when the performing harness actually measured them —
absent beats estimated, because fabricated effort data poisons the
empirical well. Two honest uses of the data: same-chore regression
detection across runs ("the dependency refresh used to take ten minutes,
now ninety — split or automate it"), and population-level cost auditing
("this chore is a large share of total maintenance spend and may deserve
an audit") — the latter flags magnitude, never cross-chore comparison of
unlike work. The population audit can itself be a chore with its own
ledger, which is self-referential in exactly the way the model should
support.

Open points if adopted: single-file JSON artifact versus Markdown +
JSONL sidecar (the sidecar is the first multi-file payload — real
identity/discovery/doctor physics, likely a dragon); the initial effort
field vocabulary; event-triggered chores ("when task X lands, do Y") are
deliberately excluded from v1 — those are triggers, not staleness.

## Evidence

Taxonomy gap observed during real work (README rework, 2026-07). JSONL
payloads are promised by decision 3 but exercised by nothing. Fortune
(idea 6) supplies the read-side channel this needs and gains a second
artifact class to grumble about. Prior art: `anacron` (tolerance-based,
not schedule-based, recurrence), aviation and equipment maintenance
logbooks (append-only ledgers with staleness tolerances and sign-offs),
oil-change stickers, MOTD nagging.
