---
id: drg_01KY169X7W0YXJ5QFV4D1MK4FB
sequence: 3
kind: dragon
status: open
created: 2026-07-20
---

# Reference marker syntax and typed edge vocabulary

## Context

Decision 0006 (`dec-bootstrap-reference-model`) settled reference
semantics — stable-ID targets with frozen labels, write-time binding,
typed edges in front matter versus untyped inline markers — but
deliberately deferred the concrete syntax as an open point.

That open point has since landed on the critical path of several
organizational answers, all of which want typed edges: excluding ideas as
dependency targets, giving invariant artifacts supersedes/amends
provenance, letting tasks carry implements edges to decisions, and letting
dragons record what resolved them. The interim prose convention works for
humans but is not machine-parseable, so no doctor check or graph
projection over references can exist until this is decided.

Update (2026-07-21): the comment-thread specimen (idea 11,
`cmt-transition-crash-contract`) surfaced a further dimension. Thread
entries wanted to be cited from canonical artifacts ("comments 1" in a
sprint amendment), but entries have no addressable identity outside
their thread file. Per-entry headings already yield tooling-free URL
fragments (`0001-thread.md#cme-entry-id` renders on GitHub), so the
grammar question now includes whether reference targets admit
sub-artifact fragments and whether the entry-heading form is part of
the reference surface.

## Question

What concrete inline marker syntax encodes untyped references, and what
initial typed edge vocabulary — with its front-matter encoding — should
canonical artifacts use? Does the target grammar admit sub-artifact
fragments (such as comment-thread entry ids), and if so, with what
orphaning semantics when the fragment's artifact is restructured?

## Constraints

- Must read acceptably in a raw pull-request diff with no tooling
  installed (decision 0007).
- Targets are stable artifact IDs with an embedded, frozen human label
  (decision 0006).
- The unbound sugar form must be a relaxation of the same grammar: legal
  but weak, repairable by an explicit bind operation.
- Typed edges live as front-matter fields; untyped markers live inline;
  nothing is inferred from unmarked prose.
- Each typed edge kind must define its doctor semantics up front (a
  dangling typed edge is likely corruption; a dangling untyped marker is a
  diagnostic at most).
- No typed dependency edge may target a non-canonical artifact such as an
  idea; provenance edges from adopting decisions are permitted.

## Candidate direction

Wikilink-style `[[id|label]]` versus Markdown-link-style
`[label](strata:id)` for inline markers; a deliberately small initial
typed vocabulary (for example supersedes, amends, resolved-by,
implements, context-from), each kind introduced only alongside its first
consumer rather than speculatively.

## Resolution criteria

A decision record fixes the inline marker syntax and the initial typed
vocabulary, passes the diff-readability test, and retires the interim
prose convention for new writing. Resolve before or alongside the first
consumer: the ideas collection (task 0006), invariant artifacts, or
doctor checks over references.
