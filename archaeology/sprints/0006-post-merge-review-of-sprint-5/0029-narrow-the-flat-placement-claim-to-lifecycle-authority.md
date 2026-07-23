---
id: tsk_01KY64ZPXPRBGH5S99G5E99TZY
sequence: 29
kind: task
status: closed
sprint: spr_01KY61D615FAC8VVSTD7QXX1DW
created: 2026-07-22
closed: 2026-07-22
---

# Narrow the flat-placement claim to lifecycle authority

## Objective

Close claim B of comment thread 7
([[cmt-s5-placement-and-cardinality|placement and sprint
cardinality]]). Decision 11 ([[dec-flat-placement|flat placement]])
claims one placement rule for every current and future collection
while its own text and the landed corpus carve exceptions: per-artifact
sprint containment directories; tasks nested inside them with no
collection directory of their own; sprint and task sharing one root;
task ownership carried in both containment and the `sprint:` field
with doctor policing agreement (`misfiled-task`, error tier) — the
doctor-as-police shape the decision cites to reject lifecycle
directories; and the provisional comment threads practicing lifecycle
placement (`comments/open/` to `comments/resolved/`, plus a `status:`
field) after the decision landed. The rejected lifecycle-directory
alternative is evaluated only in its duplicated-`status` form; the
directory-authoritative variant (directories exist, `status:` removed)
is evaluated nowhere.

The migration itself was adjudicated keep-unchanged on its merits. The
defect is the decision's overclaim and the unevaluated alternative.

## Acceptance criteria

- Decision 11 is amended (or superseded by a narrow successor) so the
  claim matches the evidence: lifecycle state is never encoded in
  canonical placement and transitions never move files; stable
  containment remains collection-specific. The migrated corpus and the
  code are untouched.
- The directory-authoritative variant is recorded among the
  alternatives with an honest evaluation rather than implied
  falsification. Thread 7 supplies the grounds found in adjudication:
  it dissolves only the double-bookkeeping ground; unstable paths and
  the empty-directory problem (dragon 2) survive; the `closed:` stamp
  makes transitions rewrite-plus-rename again for sprints and tasks,
  resurrecting the decision 8 two-step contract; and it fails
  universality on this same corpus (sprint closure either relocates
  the containment tree or keeps front-matter status). If any ground is
  wrong, the amendment records the correction instead.
- The containment/`sprint:` dual bookkeeping is acknowledged as a
  distinct accepted cost with the distinction recorded: no transition
  moves files, so tool operations cannot desync the pair — only
  hand-edits can — unlike status-in-directory, which desynced on every
  interrupted transition.
- The comment collection's placement is settled: flattened under the
  narrowed rule when comments are promoted to a managed collection, or
  recorded as a deliberate provisional exception with its promotion
  criteria named.
- The amendment passes the decision 7 raw-diff readability test;
  CLAUDE.md's conventions text is updated only where it repeats the
  overclaim.
- `scripts/check.sh` and `strata doctor` are green at close.

## Result

Closed 2026-07-22. Claim B is remediated by a dated, plainly normative
amendment appended to [[dec-flat-placement|decision 11]]; the original
text is preserved unchanged as history and its universal placement
claim is explicitly superseded. Archaeology and documentation only: no
production code, tests, README, or corpus placement changed, and the
task 18 migration was untouched (adjudicated keep-unchanged on its
merits — the defect was the claim's breadth).

**Narrowed claim.** Lifecycle state is never encoded in canonical
placement and transitions never move files; stable containment is
collection-specific rather than universally flat — dragons and ideas
happen to be flat, sprints own stable containment directories, tasks
live inside their owning sprint's containment, and sprint and task
share the `archaeology/sprints` root without pretending tasks live in
one global task directory.

**Dual bookkeeping.** Containment plus `sprint:` is recorded as a
distinct accepted cost: doctor checks their agreement
(`misfiled-task`), and the material distinction from
lifecycle-directory status duplication is recorded — neither carrier
changes during a transition, so tool operations cannot desynchronize
the pair; only hand edits or malformed writes can, unlike
status-in-directory, which could desynchronize during every
interrupted transition.

**Directory-authoritative variant.** The previously unevaluated
variant (directories exist, `status:` removed) is now honestly
evaluated on thread 7's adjudicated grounds: it dissolves only the
double-bookkeeping objection; moving paths still harm history/blame
continuity; terminal directories still hit dragon 2's empty-directory
problem; sprint/task `closed:` stamps still force rewrite-plus-rename,
restoring decision 8's two-step contract; and sprint closure either
churns every task path or retains front-matter status, defeating
directory authority. The variant loses on recorded evidence, not
implied falsification. No adjudicated ground required correction.

**Comment placement.** The `comments/open|resolved` layout is
recorded as a deliberate provisional exception while comments remain
unmanaged: its directory and `status:` duplicate lifecycle state,
tolerated temporarily for manual review operations; promotion of
idea 11 into a managed collection is the trigger to decide and
migrate stable canonical placement, a managed transition must not
preserve lifecycle-directory movement without a new explicit
decision, and manual moves continue under the documented convention
until then.

**Alignment.** CLAUDE.md's conventions bullet — the only text
repeating the overclaim — now states lifecycle authority and
collection-specific stable containment without claiming every
collection is directly flat; no unrelated prose was touched. The raw
diff is one appended amendment plus one narrowed bullet (decision 7
readability holds). Verification: complete suite 345 tests green,
`strata doctor` 60 artifacts no problems, `scripts/check.sh` passes.
