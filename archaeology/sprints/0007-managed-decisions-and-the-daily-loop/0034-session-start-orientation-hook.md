---
id: tsk_01KY7S6QB3EZY0A441WRG301FX
sequence: 34
kind: task
status: pending
sprint: spr_01KY7S6Q69YJ6HATZB48SZBRRM
created: 2026-07-23
---

# Session-start orientation hook

## Objective

Wire a SessionStart hook into this repository's checked-in Claude Code
settings that opens each session with orientation — active sprints
with their pending tasks, plus one `strata fortune` line — built only
from existing strata commands. The hook is deliberately an instrument:
its recorded friction is the evidence base for or against
[[ide_01KY7S6GHMQ8ZWNXPX7TX21X7N|idea 24]]'s `strata status`.

## Acceptance criteria

- A checked-in project settings hook runs on session start and prints
  active-sprint status and a fortune produced by strata commands.
- The hook degrades gracefully — silent or a one-line notice — when
  the binary cannot be built or run.
- Observed friction (latency, aggregation gaps, formatting, anything
  hand-stitched around missing tool support) is recorded in this
  task's result as desire-path evidence.
