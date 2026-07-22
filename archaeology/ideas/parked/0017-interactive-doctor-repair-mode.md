---
id: ide_01KY5VCC68D95MHVNC7BFFRS6G
sequence: 17
kind: idea
status: parked
created: 2026-07-22
---

# Interactive doctor repair mode

## Problem

Doctor diagnoses precisely and then walks away: every error message
ends in "repair the file by hand". For some finding classes the repair
is mechanical once a human supplies the one bit the tool cannot infer
— a lifecycle mismatch is fixed by either moving the file or rewriting
its `status`, and only the user knows which belief is true. Today that
disambiguation happens with no guardrails at all: the user hand-edits
canonical bytes to fix a state the tool understands better than they
do, outside the safe-write path, with a second doctor run as the only
net.

## Sketch

An interactive repair mode (`strata repair`, or `doctor --fix`) that
walks error findings one by one, and for each finding class whose fix
is ambiguous, poses the disambiguating question with the concrete
options and their effects:

- lifecycle mismatch: "believe the filesystem" (rewrite `status` to
  match placement) or "believe the front matter" (move the file, i.e.
  complete the transition). The prompt can carry an informed default:
  [[dec-mutation-failure-classes|decision 8]] made the status rewrite
  the commit point, so a mismatch produced by an interrupted
  transition is authoritatively resolved by front matter — but the
  tool cannot distinguish that provenance from a hand-edit, so it
  recommends rather than assumes.
- filename/front-matter sequence disagreement: the same two-belief
  shape (rename the file, or rewrite `sequence`).
- unbound sugar edges: offer the bind — which overlaps
  [[idea-links-bind-command|idea 1]]; repair mode may simply be the
  interactive surface that invokes the same bind operation.
- duplicate sequences from branch merges
  ([[drg-bootstrap-branch-collisions|dragon 1]]): renumbering is the
  reserved explicit repair operation of
  [[dec-bootstrap-reference-model|decision 6]], legal only through the
  safe-write path as one reviewable diff — exactly the contract this
  mode would inherit.

Principles, mostly already settled elsewhere: repair never guesses
(the transition machinery's "never silently repairs" stance, now with
a place to send the user); every mutation goes through the existing
safe-write/transition primitives, never ad-hoc edits; each accepted
fix is one auditable step, re-validated by the same read pipeline;
declining every prompt leaves the repository byte-identical. Findings
with an unambiguous single fix could offer plain confirmation rather
than a choice; findings the tool cannot fix (duplicate ids needing a
human decision about which artifact is which) stay report-only. This
is also where `--no-interactive` and `--dry-run` first acquire the
meaningful behavior CLAUDE.md requires before those flags may exist:
non-interactive mode applies only unambiguous fixes or none, and
dry-run prints the prompt plan.

## Evidence

Owner proposal (2026-07-22), immediately after sprint 4 landed the
severity tiers: doctor now distinguishes corruption from repairable
advice, which is precisely the metadata a repair mode dispatches on.
Every error message already ends in "repair the file by hand" —
`strata` watching users perform mechanical fixes is the same friction
class as the hand-performed transitions sprint 2 absorbed. Decision 6
reserved "an explicit repair operation" for label rewrites and
collision renumbering; this idea gives that reservation its command
surface. Prior art: `git rebase --continue`'s resolve-one-thing-at-a-
time loop, `cargo fix`'s split between auto-applicable and suggested
fixes, and interactive `fsck` as the canonical believe-which-copy
prompt.
