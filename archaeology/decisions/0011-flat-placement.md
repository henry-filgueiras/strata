---
id: dec-flat-placement
sequence: 11
kind: decision
status: accepted
created: 2026-07-22
---

# Flat placement: front matter is the only lifecycle authority

## Context

The repository entered sprint 5 carrying three placement patterns.
Dragons and ideas file under lifecycle subdirectories (`open/`,
`closed/`, `parked/`, ...); decisions share one heterogeneous
directory where accepted and any future superseded records sit side by
side; tasks split into per-sprint `pending/` and `closed/`. Sprint 4's
retrospective named the resulting blocker: tasks nest under per-sprint
directories, which the one-directory-per-state model cannot express,
and managing sprints and tasks — the last hand-performed archaeology —
was gated on resolving that layout question.

Henry's review of the sprint 5 pitch reframed the question: the
lifecycle-subdirectory pattern is the odd one out, not the norm.
Decisions have been flat and heterogeneous since sprint 1 with zero
felt pain — the strongest available evidence that directories need not
encode state. Meanwhile the subdirectory pattern carries real,
recurring costs: `status` lives in two places, so `doctor` polices
their agreement; transitions must move files, so decision 8
([[dec-mutation-failure-classes|mutation failure classes]]) needed a
two-step contract with a diagnosable torn state; moved files break
`git log` and `blame` continuity unless every caller remembers
`--follow`; and empty lifecycle directories are untrackable by Git
(dragon 2, [[drg-bootstrap-git-round-trip|git round-trip]]), forcing a
created-on-first-use convention.

## Decision

One placement rule for every current and future collection: **all
artifacts of a collection live directly in that collection's
directory, and lifecycle state is carried only in front matter.**

The canonical layout becomes:

```text
archaeology/
├── decisions/     # already flat; unchanged
├── dragons/       # open/ and closed/ merge into here
├── ideas/         # parked/, adopted/, rejected/ merge into here
├── logs/
└── sprints/
    └── NNNN-name/ # containment, not lifecycle
        ├── sprint.md
        └── NNNN-task.md ...  # pending/ and closed/ merge into here
```

Per-sprint directories survive, re-founded as pure **containment**: a
sprint's directory scopes which tasks belong to it, exactly as
`archaeology/` scopes which artifacts belong to Strata. Containment
never changes over an artifact's lifecycle, so it carries no state and
requires no moves.

Consequences for the machinery:

- **Transitions rewrite one file in place.** The decision 8 two-step
  contract (status-rewrite-first, then rename) is retired along with
  the interruption window it existed to diagnose: a transition is now
  a single safe write — stage, atomic rename over the original. The
  failure-class taxonomy itself is unchanged and still governs any
  future mutation that spans steps or artifacts; transitions simply
  stop being one of those.
- **`doctor`'s status/placement agreement check is retired** — the
  failure it policed can no longer be expressed. Its successors, both
  in the error tier: an artifact file must sit directly in its
  collection's directory (no strays in retired lifecycle
  subdirectories), and a task's `sprint:` field must name an existing
  sprint whose containment directory holds the file. Unknown or
  malformed `status` values remain errors, unchanged.
- **Stable identity and sequence rules are untouched** (decision 2,
  [[dec-bootstrap-stable-identity|stable identity]]): flat placement
  changes paths only; identities, sequences, and the no-reuse and
  no-renumber rules do not move.
- **The created-on-first-use convention for lifecycle directories is
  moot** — there are no lifecycle directories. Collection directories
  are created by `init` or first `new`; sprint containment directories
  by `new sprint`. Dragon 2's underlying constraint (Git cannot track
  empty directories) still holds and still forbids pre-creating empty
  containment directories.

## Accepted cost

Status-filtered queries — "open dragons", "parked ideas", the most
common operations — must now read front matter for the whole
collection, terminal long tail included. At current scale this is
microseconds; the no-tooling test still passes, since
`grep -l 'status: open'` replaces `ls open/`. What is genuinely lost
is browse-time glanceability: a directory listing no longer shows
state. `strata list` is the answer the tool gives; if scan cost is
ever felt in a large repository, the parked counter-lever is a
modification watermark ([[ide_01KY5X7C56KBFWJJJKHTEXXQXV|idea 18]]),
a disposable projection, deliberately not built now.

## Alternatives rejected

- **Lifecycle subdirectories everywhere** (align on the dragon/idea
  pattern): double bookkeeping with `doctor` as permanent police,
  unstable paths, the empty-directory problem, and no answer for
  nested tasks — the pattern that cannot express the corpus loses to
  the one that already does.
- **Front-matter-driven recursive placement** (user-declared placement
  rules computed from arbitrary fields): maximally flexible and
  superficially aligned with user-defined collections (idea 10), but
  it makes every scan interpret a rule language, turns every rule
  change into a migration, and encodes as canonical layout what is
  really a query — the tool can compute any grouping as a disposable
  projection without moving a single file.
- **Keep the heterogeneous status quo**: three patterns forever, in
  code, docs, and every future collection decision. Rejected on the
  evidence that motivated this record.

## Consequences

- Task 18 migrates the corpus (`git mv`, no content rewrites beyond
  paths) and aligns dragon and idea code, in one slice, because
  `doctor` enforces placement and would reject a corpus that moved
  ahead of the code.
- Sprints and tasks become manageable collections with no special
  placement carve-out: `sprint.md` transitions in place like
  everything else.
- CLAUDE.md's layout and conventions are updated by the same task that
  records this decision; the status-equals-directory-name convention
  is retired for new writing immediately.
