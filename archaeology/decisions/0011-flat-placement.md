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

## Amendment: narrowed to lifecycle authority (2026-07-22)

Thread 7 claim B ([[cmt-s5-placement-and-cardinality|placement and
sprint cardinality]], owed by
[[tsk_01KY64ZPXPRBGH5S99G5E99TZY|task 29]]) found that this record's
universal claim — "one placement rule for every current and future
collection: all artifacts of a collection live directly in that
collection's directory" — is contradicted by its own text and the
landed corpus: sprints use per-artifact containment directories, tasks
nest inside them with no collection directory of their own, sprint and
task share one root, and task ownership is carried in both containment
and the `sprint:` field with doctor policing agreement. The original
text above is preserved unchanged as history; this amendment
supersedes its universal placement claim. The migrated corpus and the
code are untouched — the migration was adjudicated keep-unchanged on
its merits; the defect was the claim's breadth.

### The surviving rule

- Lifecycle state is never encoded in canonical placement, and
  lifecycle transitions never move files. This is the load-bearing
  content of this decision, and it stands.
- Stable containment is **collection-specific**, not universally
  flat. Dragons and ideas happen to be flat; sprints own stable
  containment directories that never change over their lives; tasks
  live inside their owning sprint's stable containment. Sprint and
  task share the `archaeology/sprints` root — the layout does not
  pretend tasks live directly in one global task directory.

### Accepted cost: containment plus `sprint:` dual bookkeeping

A task's ownership is recorded twice — by the containment directory
holding its file and by its `sprint:` front-matter field — and doctor
checks their agreement (`misfiled-task`). This duplication is
accepted, and it differs materially from the lifecycle-directory
status duplication this decision retired: neither carrier changes
during a lifecycle transition, so tool-driven transitions cannot
desynchronize the pair. Only hand edits or malformed writes can.
Status-in-directory, by contrast, could desynchronize during every
interrupted transition, which is why it needed decision 8's two-step
contract and a diagnosable torn state.

### The directory-authoritative variant, evaluated

The alternatives above evaluate lifecycle subdirectories only in
their duplicated-`status` form. The missing variant — lifecycle
directories exist and the `status:` field is **removed**, making the
directory the sole authority — is evaluated here rather than left
implicitly falsified:

- it genuinely dissolves the double-bookkeeping objection: with one
  carrier there is no agreement for doctor to police;
- paths still move on every transition, harming `git log`/`blame`
  continuity unless every caller remembers `--follow`;
- terminal-state directories still hit the empty-directory problem
  (dragon 2): Git cannot track an emptied `open/`;
- sprint and task `closed:` stamps still require a content rewrite,
  so their transitions become rewrite-plus-rename again, restoring
  exactly the decision 8 two-step returned-error/interruption
  contract that in-place rewriting retired;
- it fails universality on this same corpus: closing a sprint would
  either relocate its entire containment tree — churning every task
  path — or retain front-matter status for sprints, defeating
  directory authority.

The variant therefore still loses, but on recorded evidence rather
than implied falsification.

### Comment placement: a deliberate provisional exception

Comment threads currently practice lifecycle placement under
`archaeology/comments/open/` and `archaeology/comments/resolved/`,
with a `status:` field duplicating the directory. This is recorded as
a deliberate provisional exception, not a generalization of the
canonical placement rule: comments remain **unmanaged** specimens
(provisional test data for ideas 11 and 19), and the layout is
tolerated temporarily because manual review operations lean on
browse-time glanceability. Promotion of idea 11 into a managed
collection is the trigger to decide and migrate stable canonical
placement; a managed transition must not preserve lifecycle-directory
movement without a new explicit decision. Until promotion, manual
moves continue under the documented convention.
