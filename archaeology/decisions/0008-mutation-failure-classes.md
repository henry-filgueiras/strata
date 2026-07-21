---
id: dec-mutation-failure-classes
sequence: 8
kind: decision
status: accepted
created: 2026-07-21
---

# Mutation safety is scoped by failure class

## Context

External review of the sprint 2 lifecycle-transition contract (thread
`cmt-transition-crash-contract`, the first comment-thread specimen for
idea 11) objected that "at every failure point the artifact exists at
exactly one path with valid contents" promises a transaction portable
filesystems cannot provide: a transition changes both a file's path and
its contents, and no portable primitive spans a content rewrite and a
directory-crossing rename atomically.

The objection's core is correct. Its failure taxonomy, however, merged
two classes that behave very differently, and the guarantee that
survives each class is worth settling once, for every current and
future mutation, rather than per task.

## Decision

Every mutation contract names its guarantees per failure class:

1. **Returned errors** — the strongest class, fully exercisable by
   fault-injection tests. The operation leaves exactly one valid
   artifact: unchanged where possible. A multi-step mutation that fails
   partway compensates (rolls back its completed steps); only a
   doubly-degraded environment — the rollback itself failing — may
   leave an intermediate state, and the error message must then name
   that state explicitly.
2. **Abrupt process termination, OS surviving** (kill, panic, OOM).
   Every individual step must be a single atomic rename, so at every
   instant exactly one path holds valid contents — never zero copies,
   never two. Cross-step invariants (front-matter `status` agrees with
   lifecycle placement) may be caught mid-flight; the resulting state
   must be precisely diagnosable by `doctor` without guessing, and
   re-runnable tooling or an explicit repair completes or reverts it.
3. **Power loss and kernel crashes** — excluded for bootstrap, as the
   sprint 1 write contract already documented: nothing is fsynced, so
   even a single rename-based replace may surface torn after an
   unclean shutdown. Dragon 4 (`drg_01KY3C0S3JQKEMEB9BH6NVJ35F`) owns
   whether this class is ever brought in scope.

Two further rules for multi-step mutations:

- **The content rewrite is the commit point.** A transition orders its
  steps status-rewrite-first, then rename: placement follows status.
  The artifact's own front matter is the authored, richer record; the
  directory is mechanical filing. An interrupted transition therefore
  reads as "committed but not yet filed", and a future repair operation
  has a deterministic rule — trust the front matter, fix the placement.
  The reverse ordering would leave the file's own metadata stale and
  make the less expressive property (placement) the authority.
- **No transition journal.** The interruption window is two syscalls
  wide, touches one artifact, and its degraded state is fully
  diagnosable and repairable from canonical files alone. A journal is
  operational state in tension with decision 1 (files are canonical)
  and is unjustified until a mutation spans multiple artifacts; that is
  the revisit trigger.

`doctor` deliberately does not distinguish an interrupted transition
from a hand-edit that changed only `status`: the on-disk states are
identical, so any distinction would be a guess. One mismatch diagnosis
naming both sides is the honest report, and the existing finding
already provides it.

## Consequences

- Sprint 2 and task 7 acceptance criteria are amended to this contract;
  the original single-sentence guarantee is superseded.
- Fault-injection tests cover class 1 boundaries; class 2 is tested by
  constructing each intermediate state directly on disk and pinning its
  `doctor` diagnosis — deterministic, unlike kill-timing tests.
- A rename that fails across filesystems (EXDEV) or on a read-only
  destination is a class 1 failure: rolled back, artifact intact at the
  source.
- Future mutations (repairs, batch operations) inherit this taxonomy;
  promising class 2 or 3 guarantees beyond it requires a new decision
  with fsync and recovery design in hand.
