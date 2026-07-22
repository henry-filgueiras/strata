---
id: tsk_01KY640RFXZJMWZ2T8W9B628AA
sequence: 27
kind: task
status: pending
sprint: spr_01KY61D615FAC8VVSTD7QXX1DW
created: 2026-07-22
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
