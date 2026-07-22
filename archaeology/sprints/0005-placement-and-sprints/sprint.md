---
id: spr-placement-and-sprints
sequence: 5
kind: sprint
status: active
created: 2026-07-22
---

# Sprint 5: One placement model and managed sprints

## Goal

Fix a single placement model for all collections by decision — the
leading candidate is flat per-collection directories with lifecycle
state carried only in front matter — migrate the corpus and the code to
it, then make sprints and tasks the third and fourth CLI-managed
collections so that opening a sprint, adding tasks, closing a task with
its result, and closing a sprint are intent commands. Let provenance
ride the transition commands that already exist.

## Rationale

Sprint 4's retrospective names sprint and task closure as the last
recurring hand-performed archaeology, for the third consecutive sprint.
The named blocker was layout: tasks nest under per-sprint directories,
which the one-directory-per-state model cannot express. Henry's
observation while reviewing this pitch dissolved the blocker from the
other side: the repository already carries three placement patterns —
dragons and ideas in lifecycle subdirectories, decisions in one
heterogeneous directory, tasks in per-sprint pending/closed splits —
and the lifecycle-subdirectory pattern is the odd one out, not the
norm. Status-in-directory is double bookkeeping: `doctor`'s
status/placement agreement check exists only because state lives in
two places, and the two-step transition contract of decision 8
([[dec-mutation-failure-classes|mutation failure classes]]) exists
only because transitions move files. Flat placement retires the
failure class instead of policing it, keeps artifact paths stable so
`git log` and `blame` work without rename detection, and renders a
state change as a one-line front-matter hunk beside its result append.
The known price — status filters must read front matter for the whole
collection, terminal long tail included — is accepted at current scale
and has a recorded counter-lever
([[ide_01KY5X7C56KBFWJJJKHTEXXQXV|modification watermark]]) if it ever
hurts.

This sprint's own artifacts are deliberately minted under the old
convention (this file's siblings sit in `pending/`), because the
decision that retires that convention has not been made yet; the
migration task sweeps sprint 5 itself, making the sprint its own test
data.

A third and fourth managed collection again widen the exposure of
dragon 1 ([[drg-bootstrap-branch-collisions|branch sequence
collisions]]); this is accepted unchanged, as sprint 4 accepted it.
Dragon 4 ([[drg_01KY3C0S3JQKEMEB9BH6NVJ35F|power-loss durability]]) is
unaffected in kind, though in-place transitions narrow each mutation
to a single file.

## Success criteria

A user can run:

```sh
strata new sprint "Some goal"
strata new task "Some work item"
strata list sprints
strata list tasks
strata list tasks --json
strata show task:17
strata close task:17
strata close sprint:5
```

The sprint must deliver:

- a decision record fixing one placement model for every collection —
  directory layout, front-matter authority for status, `doctor`
  semantics replacing the status/placement agreement check, the fate
  of the decision 8 two-step transition contract, and the recorded
  alternatives (front-matter-driven recursive placement, keeping the
  status quo) — passing the raw-diff readability test of decision 7
  ([[dec-bootstrap-interaction-surfaces|interaction surfaces]]), with
  CLAUDE.md's layout and conventions updated to match;
- the existing corpus migrated to the decided model with history
  preserved (`git mv`, no identity or sequence rewrites per decision 2,
  [[dec-bootstrap-stable-identity|stable identity]]), dragon and idea
  code aligned, and `doctor` green after every step;
- sprints and tasks managed end to end: creation with safe numbering
  and generated stable identity, discovery, listing (human and
  `--json`), show, and closure under the decision 8 failure-class
  contract, with hand-seeded `spr-*` and `tsk-*` identities remaining
  valid and never rewritten; closing a sprint with pending tasks is
  refused;
- the third-and-fourth-collection duplication verdict recorded as
  evidence for idea 10
  ([[idea-declarative-collection-specs|declarative collection specs]]);
- transition commands accept provenance for the existing edge
  vocabulary (`close --resolved-by`, `adopt --adopted-by`), writing
  edge and transition in one invocation;
- `doctor` green on this repository after every retrofit.

## Non-goals

- The spec engine itself (idea 10): the rule-of-three evidence is
  complete after this sprint; extraction is the natural candidate for
  sprint 6, not a stowaway here.
- The modification watermark
  ([[ide_01KY5X7C56KBFWJJJKHTEXXQXV|idea 18]]): a perf seam, parked
  until a real repository demonstrates felt scan cost.
- Fortune drawing from pending tasks: the pool stays open dragons and
  parked ideas; widening it is a deliberate follow-up decision.
- New typed edge kinds: only the decided vocabulary gains a command
  surface; nothing new is introduced without a consumer (decision 10
  rule).
- `strata links bind` (idea 1), reference-graph projections (idea 2),
  cross-sprint dependency enforcement (idea 14), strict doctor
  (idea 13).
- Bulk migration of historical prose references: unchanged policy from
  decision 10 — retrofits only where a task already rewrites the file.
