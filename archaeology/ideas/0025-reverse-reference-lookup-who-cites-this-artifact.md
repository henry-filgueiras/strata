---
id: ide_01KY7S6GK44ZVC8X2CF4KCM8MD
sequence: 25
kind: idea
status: parked
created: 2026-07-23
---

# Reverse reference lookup: who cites this artifact

## Problem

Markers make the corpus a graph, but only forward edges are readable: a
file names what it cites. The load-bearing question usually runs
backwards — before amending a decision or closing a dragon, what cites
it and would be invalidated by the change? Today the answer is a grep
over marker syntax, which requires knowing the target's stable id *and*
any legacy id *and* every unbound sugar form that might still name it —
exactly the resolution work the read side already implements once,
correctly, in the claimant catalog.

## Sketch

`strata links to <ref>` (final name open) lists every artifact
containing a marker that resolves to the target, with the containing
line for context, resolving through the same claimant catalog the read
side uses. Read-only, nothing persisted, scan per invocation at
current scale.

Stepping stones: [[idea-doctor-reference-graph|doctor's reference-graph
checks]] need the same inverted view transiently;
[[idea-proposal-relevance-surfacing|relevance surfacing]] needs at
least backlinks before it can rank anything; the
[[idea-links-bind-command|bind command]]'s check mode shares the marker
scanner. This is the smallest of the four and the only one with no
policy content.

## Boundaries

- Derived, never canonical; no stored index.
- Reports, does not validate: dangling markers remain doctor's
  business per [[idea-doctor-reference-graph|idea 2]].
- Resolution semantics are exactly the read side's; no second
  resolver.

## Evidence

Sprint 6's adjudication repeatedly needed "which artifacts reference
this" while narrowing decisions 11 and 14, answered by manual grep
each time. The corpus's seven dated update/amendment sections each
faced the who-depends-on-the-old-text question with no tool support.

Proposed by Claude during the sprint 7 pitch, 2026-07-23.
