---
id: tsk_01KY640RFXZJMWZ2T8W9B628AA
sequence: 27
kind: task
status: closed
sprint: spr_01KY61D615FAC8VVSTD7QXX1DW
created: 2026-07-22
closed: 2026-07-22
---

# Corpus operability policy under malformed siblings

## Objective

Close case G of comment thread 6 (supplemental round). Creation and
the ordinary reads apply incompatible corpus policies to the same
managed directory: sequence allocation is filename-only and
content-blind (pinned by test — it tolerates arbitrary junk content),
while `list`, `show`, and the transition resolvers strongly parse
every sibling and abort on the first malformed one. Composed, `strata
new dragon` beside a malformed sibling reports success and prints a
reference (`created dragon:2 at …`) that no read or transition command
can resolve until the *unrelated* sibling is repaired — reproduced:
`show dragon:2`, `show <id>`, `list dragons`, and `close dragon:2` all
exit 5 naming only the sibling, while the new artifact is individually
valid and fully operable the moment the sibling is removed.

Each boundary is individually deliberate and documented; nothing
documents their composition. The repository was already doctor-red
before creation and creation adds no finding, so this is not a
mutation-corrupts case: it is a missing degraded-mode contract.

Constraint discovered during adjudication: the strict scan is
currently the guard that surfaces a malformed *duplicate claimant* of
a requested sequence (a malformed file claiming `sequence: 2` beside a
valid `dragon:2` makes resolution refuse rather than silently pick the
valid one). Any isolation policy must preserve that refusal, and the
same masking concern applies to id resolution — this seam touches
task 23's ambiguity classification.

Closure property from the thread, as extended by the supplemental
round: property 2 (doctor-green implies operable) must hold per
artifact even when a *sibling* is red, or the policy must say
explicitly that it does not and make creation's success reporting
honest about it.

## Acceptance criteria

- A recorded decision states the corpus operability policy under
  malformed siblings, choosing among (at least): creation refuses when
  the collection cannot be strongly scanned; ordinary commands isolate
  malformed siblings while doctor reports them; a documented degraded
  mode in which creation's successful result is nevertheless showable
  and operable. The decision records why the losing candidates lose.
- Whichever policy is chosen, the composition invariant holds: a
  creation that reports unqualified success yields an artifact that
  `show` (by sequence and by stable id), `list`, and its admitted
  lifecycle transitions can reach — or creation's output explicitly
  qualifies the degraded state and names the blocking sibling.
- Sequence-allocation collision safety is preserved exactly: malformed
  siblings still occupy their sequences, sequences are never reused,
  and resolution never silently bypasses a malformed file that could
  claim the requested sequence or id (the duplicate-claimant guard).
  Alignment with task 23's identity catalog is recorded — a malformed
  claimant is ambiguity evidence, not skippable noise.
- When a command cannot reach an otherwise valid target because of a
  malformed sibling, the diagnostic says so: it names the target it
  could not deliver as well as the sibling that blocked it, rather
  than reporting only the sibling.
- Regression tests cover the adjudicated matrix: creation beside a
  malformed sibling; show by both spellings; list; an admitted
  transition; doctor before and after; recovery after removing only
  the sibling; and the malformed-duplicate-claimant refusal.
- Prompt 5's read-architecture work may reuse this seam, but this
  contract lands on its own evidence and does not wait for it.
- `scripts/check.sh` and `strata doctor` are green at close.

## Result

Closed 2026-07-22. Case G is remediated by implementing
[[dec-degraded-corpus-operability|decision 13]] exactly as recorded;
the decision was not amended.

**Composition invariant.** A creation that reports unqualified success
yields an artifact `show` (by sequence and stable id), `list`, and its
admitted transitions can reach; otherwise the output explicitly
qualifies the degraded state and names the blocking sibling. Removing
only the blocker restores full reachability with no repair to the
created artifact.

**Observational probe.** `artifact::probe_reachability` runs after a
successful flat write (`new dragon`, `new idea`): it replays the
normal strict read path and proves resolution by both sequence and
stable id. It never rolls back the write, never mutates, and never
becomes a nonzero exit; the strict scan's first failure is by
construction the deterministic blocker. Sequence allocation remains
filename-only and content-blind — a malformed sibling occupies its
sequence and blocks nothing.

**Warning contract.** Degraded creation emits exactly one stable
stderr line beginning `warning[degraded-repository]:`, naming the
created reference and path, the blocking sibling path with its
underlying reason, that the artifact was created with exit status
success, and that repairing the blocker restores normal access. The
`error[...]` token is never used for this successful state.

**JSON schema.** `strata new --json` now exists for all four managed
kinds (decision 13 requirement, not a placeholder): stdout is one
deterministic object — `kind`, `id`, `sequence`, `reference`, `path`
(root-relative) — identical for healthy and degraded creation, with
the warning on stderr leaving stdout parseable. Human and JSON paths
consume the same `NewArtifact` + `Reachability` semantic result.

**Strict diagnostics.** Reads stay strict; nothing skips malformed
siblings. `Error::blocking` attaches the requested target to a
sibling-blocked scan in `show`, `close`/transitions (including the
bare-id union scans), preserving the original typed category while
naming both the undeliverable target and the blocking sibling; files
are left unchanged. `list` remains a strict whole-collection scan
naming the blocker. Malformed duplicate sequence/id claimants remain
refused evidence (task 23 alignment) — the strict scan still surfaces
them, and the probe introduces no bypass.

**Flat versus containment.** Sprint/task creation retains its strict
pre-write scans and may fail without creating anything; no global
repository-valid flag, and the probe is one blocker's observation,
not a second doctor.

**Evidence** (`tests/degraded.rs`, 13 tests): allocation past a
malformed sibling for dragon and idea; qualified human output;
parseable degraded JSON with the stderr warning; exit 0; no new
doctor finding from creation; show refusals by both spellings naming
target and blocker; strict list; refused transition leaving bytes
unchanged; blocker-removal recovery without touching the artifact;
duplicate sequence and id claimants never bypassed; healthy creation
warning-free; sprint/task strict boundary. Complete suite 338 tests
green; `strata doctor` 60 artifacts, no problems; `scripts/check.sh`
passes; task 23 claimant and task 24–26 regressions green.
