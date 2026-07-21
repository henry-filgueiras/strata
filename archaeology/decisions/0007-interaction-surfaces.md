---
id: dec-bootstrap-interaction-surfaces
sequence: 7
kind: decision
status: accepted
created: 2026-07-20
---

# Raw narrative Markdown stays first-class; sugar lives on the write path

## Context

A recurring design temptation frames Strata as the "browser" for its
repository, demoting raw Markdown to a view-source representation that
users are not really expected to read. This would quietly erode the core
invariant that a Strata repository remains understandable and editable
without the Strata executable.

The reading surfaces that matter most are outside Strata's control and
render raw source: pull-request diffs (the review and audit surface), merge
conflict resolution, agent retrieval over plain files, forge web views, and
eventual archaeology performed after the tool itself is gone.

## Decision

- For narrative Markdown artifacts, the raw file is a first-class reading
  surface, permanently. Ergonomic and syntax proposals are evaluated
  against how they read in a raw PR diff with no tooling installed;
  proposals that fail that test are rejected or redesigned (embedding a
  human label beside a stable ID in references is the canonical example of
  passing it).
- Interactive sugar — completion, reference binding, link following,
  editor affordances — belongs on the write path, as progressive
  enhancement over an already-readable source. Editors integrate as thin
  shims over the Strata core so human and machine callers share one code
  path; resolution and validation logic never lives in editor code.
- An edit-through-projection flow (open a projection in `$EDITOR`,
  validate and bind on return, in the style of `git rebase -i` or
  `kubectl edit`) is an approved future pattern under two constraints:
  the projection must be a relaxation of the canonical grammar (decision
  0006), and content the user did not touch must round-trip byte-identical
  so edit sessions never generate formatting churn. Editor-return is also
  the right moment to run artifact validation and refuse malformed
  results without losing the user's text.
- Structured payloads (JSON, JSONL — decision 0003) are exempt: they are
  legitimately projection-first, and tool-rendered views may be their
  primary human interface.
- Tooling never rewrites prose outside an explicit, user-initiated
  operation whose diff the user can review. No save hooks, no CI commits
  (automatic commits are already a recorded non-goal), no batch rewrites
  hidden inside unrelated commands.

## Consequences

- The repository stays legible to reviewers, mergers, agents, and future
  excavators who have nothing installed — the audiences the paper-trail
  mission exists for.
- Choosing Markdown keeps paying its dividend: the source is the artifact,
  as that format intends.
- Write-side features accumulate against one core grammar instead of
  accreting dialects; the cost is that some ergonomic ideas must be
  redesigned (or rejected) when they only work in a rendered view.
- Read-side projections may freely display current truth (for example,
  refreshed reference labels per decision 0006) precisely because they are
  disposable and the canonical file is not.
