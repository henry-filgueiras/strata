---
id: tsk_01KY6364DMJ7DPEWCAK0ZKDNHR
sequence: 24
kind: task
status: closed
sprint: spr_01KY61D615FAC8VVSTD7QXX1DW
created: 2026-07-22
closed: 2026-07-22
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

One shared validator, `artifact::validate_title`, now runs first in every
creating path: `create` (dragons and ideas), `create_sprint`, and
`create_task` all call it on the raw supplied title before `trim()`,
slugging, corpus scans, sequence allocation, ULID generation, rendering,
or directory materialization. It rejects exactly the characters
`char::is_control` admits — LF, CR, tab, NUL, DEL, and the remaining
Unicode Cc controls — as `invalid-invocation` (exit 2), naming the
single-line/no-control-character constraint and the first offending
character by escaped spelling and code point (e.g. `` `\n` (U+000A) ``);
no raw control character is interpolated into stderr, and the diagnostic
is one line. Nothing is sanitized or discarded: the invocation is
refused with every repository file and directory untouched. Ordering is
pinned by refusals that outrank later checks: a control-bearing task
title refuses before the no-active-sprint error, and a control-bearing
sprint title refuses before the one-active-sprint refusal
(`every_creator_validates_the_title_before_other_work`). Leading and
trailing controls are refused rather than hidden by the later trim.
Trim, the ASCII-slug requirement, Unicode titles, and
marker-significant punctuation (`#`, `|`, `]`, `]]`) are unchanged as
title content (`marker_significant_punctuation_remains_legal_title_content`);
task 25's frozen-marker rules govern any later binding of such titles.

Sprint creation now honors decision 8's returned-error class:
`create_sprint` records the root-relative directories `ensure_dir`
actually created and, on any later failure — including an `ensure_dir`
failure after partial materialization or a `write_new` failure —
`rollback_sprint_dirs` removes exactly those paths in reverse creation
order using `fs::remove_dir` (empty-directory removal only). A
pre-existing directory is never in the created list, and `remove_dir`
refuses non-empty directories, so pre-existing parents and concurrent
content are never deleted to make rollback pass
(`rollback_preserves_preexisting_directories_and_the_retry_reuses_the_sequence`,
`obstructed_rollback_is_a_filesystem_failure_naming_original_and_leftover`).
When cleanup succeeds the original typed error returns unchanged,
category and diagnostic intact; when cleanup itself fails — decision 8's
doubly degraded case — the existing `filesystem-failure` category names
the original creation failure, the exact path whose cleanup failed, and
that structural debris may remain and requires inspection. No new
frozen error category was added. The fault-injection seam is a private
`create_sprint_with(root, title, write)` whose production caller always
passes `write_new`; no CLI flag, environment variable, or `umask`
mutation exists. After a rolled-back failure, an ordinary `new sprint`
retry succeeds reusing the still-available display sequence. `strata
init`'s nontransactional directory creation (decision 5) and flat
dragon/idea materialization are unchanged — rollback is local to sprint
artifact creation.

Round-trip evidence: one artifact of each managed kind created through
the normal binary surface (sprint before its task, respecting the
singleton rule) resolves through `show` and leaves doctor green at
"4 artifact(s) checked, no problems found"
(`each_kind_round_trips_through_show_and_doctor_stays_green`). No
runtime doctor scan was added to creation; the template plus validator
establish validity and the round-trip tests pin it.

Boundaries preserved: line endings and `.gitattributes` untouched
(task 26); content-blind filename-only sequence allocation, creation
beside malformed siblings, and strict read behavior untouched (task 27,
decision 13); the one-active-sprint rule and bare-task selection
untouched (task 28); task 25's id, status, and marker contracts and its
21 representation tests untouched and green.

Regression evidence: 10 focused tests added (7 unit, 3 integration);
suite now 196 unit + 110 integration tests, all green with
`scripts/check.sh` and repository doctor (60 artifacts, no problems).
