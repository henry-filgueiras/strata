---
id: idea-single-invocation-commits
sequence: 9
kind: idea
status: parked
created: 2026-07-21
---

# Single-invocation commits for state-changing commands

## Problem

A lifecycle transition is one intent but several mechanical steps: move
the file, rewrite its status, stage, commit. Performed by hand (three
times in sprint 1), each step is a chance for placement/status drift and
for the transition to land tangled into an unrelated commit — which also
muddies the blame-derived provenance that [[idea-chore-artifacts]]
depends on.

## Sketch

An opt-in flag — `--commit` on state-changing commands such as a future
`strata close` — that turns one invocation into one commit:

- preflight asserts the paths the operation will touch are clean in the
  worktree (a dirty repository is a typed error, not a merge of the
  user's half-done work into a tool commit);
- the mutation runs as normal;
- exactly the invocation's delta is staged and committed with a
  generated `area: what changed` message naming the artifact;
- push never happens; pushing stays a human decision.

The payoff is atomicity of intent: `git blame` on any line the
invocation wrote resolves to a commit describing exactly that intent,
"fewer manual steps to make bad decisions with" (the user's framing),
and crisp provenance for ledger-style artifacts.

Deliberately parked rather than tasked: automatic commits are a recorded
bootstrap non-goal, so adoption requires a new decision plus evidence —
likely the friction count from sprint 2's manual `close` workflow. It is
also a Git-aware feature and must stay optional at the core: without a
`git` binary or repository, the flag fails with a typed error and the
command works un-flagged. Editor-mediated prose at transition time
(append a resolution section via `$EDITOR`) is adjacent but separate —
see [[idea-strata-edit]] and [[idea-editor-integration-shims]].

## Evidence

Sprint 1 retrospective: three hand-performed transitions, each
move-plus-status-plus-commit. Commit policy in CLAUDE.md already
mandates the shape this flag would automate (one commit per completed
slice, archaeology included). Explicit non-goal list ("automatic
commits") is the adoption gate. Prior art: `git commit --only`,
jujutsu's operation-per-change model, database autocommit semantics.
