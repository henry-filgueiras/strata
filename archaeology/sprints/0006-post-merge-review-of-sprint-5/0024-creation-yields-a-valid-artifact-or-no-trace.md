---
id: tsk_01KY6364DMJ7DPEWCAK0ZKDNHR
sequence: 24
kind: task
status: pending
sprint: spr_01KY61D615FAC8VVSTD7QXX1DW
created: 2026-07-22
---

# Creation yields a valid artifact or no trace

## Objective

Close cases A and F of comment thread 6: intent commands can currently
leave the repository doctor-red after both a *successful* and a
*failed* creation.

Case A: `strata new` interpolates the title verbatim into the `# `
heading of the rendered template. A title containing a newline
followed by `# ` stays sluggable, so creation succeeds (exit 0) and
writes an artifact with two level-one headings, which the shared
reader — and therefore every subsequent command — rejects as
malformed.

Case F: `create_sprint` materializes the containment directory with
`ensure_dir` but discards the created-directories list, then calls
`write_new`. A returned write failure (reproduced with an unwritable
fresh directory) leaves a containment directory without `sprint.md`.
That debris is a doctor error, and because every sprint scan hard-fails
on it, it also blocks `new sprint` retries, `list sprints`, `list
tasks`, and sprint transitions until repaired by hand. This violates
the module's own guarantee that a failed creation leaves no partial
destination artifact.

Closure properties from the thread: every artifact produced by an
intent command passes doctor (1); a returned mutation error preserves
the previously valid canonical state (4).

## Acceptance criteria

- Creation refuses titles that cannot be rendered into a valid
  artifact, before writing anything: at minimum titles containing
  newlines or other control characters are an `invalid-invocation`
  naming the constraint. The refusal covers every creating command
  (dragon, idea, sprint, task) through the shared path.
- A returned failure in `create_sprint` after the containment
  directory was materialized removes the directories that call
  created (and only those), restoring the pre-call tree. A directory
  that already existed is never removed.
- Regression tests: a `\n# `-bearing title is refused with nothing
  written; an induced post-`ensure_dir` write failure leaves no
  containment debris and a subsequent `new sprint` succeeds; a
  created artifact of each kind round-trips through the reader and a
  clean doctor pass.
- `scripts/check.sh` and `strata doctor` are green at close.

## Result
