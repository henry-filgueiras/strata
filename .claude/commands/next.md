---
description: Reorient from the archaeology and pitch or continue the next unit of work
---

Reorient from the repository's own records, not from conversational
memory. This command is the standard opening move after `/clear`: the
archaeology is the handoff, so no conversational context needs to
survive the boundary.

1. Read `CLAUDE.md` in full.
2. Run `strata doctor` and `strata fortune` (build first if needed) —
   health check and ambient recall, dogfooding both.
3. Find the newest sprint under `archaeology/sprints/` (highest
   sequence) and read its `sprint.md`.
   - If it is **active**: read its pending tasks and continue with the
     first actionable one, honoring the archaeology workflow in
     CLAUDE.md.
   - If every sprint is **closed**: read the newest retrospectives
     ("friction to fix next" especially) and the parked ideas and open
     dragons they cite, then pitch the next sprint — goal, rationale,
     candidate tasks, non-goals — as a proposal for discussion. Do not
     scaffold sprint artifacts until the pitch is agreed.
4. Name the decisions and open dragons directly relevant to whatever
   you propose or continue, by reference.

If arguments are given, they narrow or override the focus: $ARGUMENTS
