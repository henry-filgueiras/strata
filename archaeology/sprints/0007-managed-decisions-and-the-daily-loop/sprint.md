---
id: spr_01KY7S6Q69YJ6HATZB48SZBRRM
sequence: 7
kind: sprint
status: active
created: 2026-07-23
---

# Sprint 7: Managed decisions and the daily loop

## Goal

Land four small, independent, unambiguous vertical slices chosen for
immediate utility: make decisions the fifth managed collection
(task 32), make `strata doctor` a commit gate in `scripts/check.sh`
(task 33), wire a session-start orientation hook into this repository
as a deliberate desire-path instrument (task 34), and ship shell
completions (task 35).

## Rationale

Sprint 5's retrospective named managed decisions the next-collection
candidate; sprint 6 deferred it behind the incident hold, which is now
released. The corpus is already conformant — all fifteen decisions
carry managed-style front matter with uniform `accepted` status — so
the slice is command coverage and discovery, not migration.

The other three items convert existing capability into daily-loop
pressure. Doctor inside `check.sh` makes archaeology validity a
commit-gate fact instead of a remembered manual step. The session-start
hook is an instrument as much as a feature: every session it runs
either confirms the orientation ritual is served by existing commands
or generates concrete friction evidence for or against
[[ide_01KY7S6GHMQ8ZWNXPX7TX21X7N|idea 24]]. Completions lower the cost
of human CLI use, which is where desire-path data comes from.

Deliberately not smuggled in: the spec-engine extraction
([[idea-declarative-collection-specs|idea 10]]) is not required by any
task here. If task 32's implementation shows that a fifth collection
means a wholesale further copy of collection mechanics, the implementer
surfaces that as a decision point rather than copying silently or
extracting silently — adoption of idea 10 is Henry's call, made on
that evidence.

A fifth managed collection widens the exposure of
[[drg-bootstrap-branch-collisions|dragon 1]] again; accepted unchanged,
as sprints 4 and 5 accepted it.
[[drg_01KY3C0S3JQKEMEB9BH6NVJ35F|Dragon 4]] is unaffected in kind.

## Success criteria

- `strata new decision`, `strata list decisions` (with `--json`), and
  `strata show decision:N` (with `--json`) work, and `doctor` validates
  decisions under the same invariants as other collections, over the
  unmodified existing corpus of fifteen files.
- `scripts/check.sh` fails when `strata doctor` reports problems in
  this repository.
- A fresh Claude Code session in this repository opens with
  active-sprint status and one fortune line produced by strata
  commands, and the friction observed is recorded in task 34's result.
- `strata completions <shell>` emits a completion script that loads
  cleanly in zsh at minimum.
- Every task closes with its result recorded; `scripts/check.sh` and
  `strata doctor` are green at close.

## Non-goals

- Decision lifecycle beyond creation as `accepted`: supersession and
  deprecation wait for the first real supersession event.
- Managed logs or comments collections.
- The spec-engine extraction (idea 10) as an end in itself; see
  Rationale for the only path by which it may enter.
- Relevance, ranking, or projection work: ideas 8, 12, 24, 25, and 27
  remain parked.
- The standing bootstrap non-goals: daemon, watcher, index,
  embeddings, MCP, GraphQL.
