---
id: ide_01KY7S6GP6MBA53MSG8JNJ2AJM
sequence: 27
kind: idea
status: parked
created: 2026-07-23
---

# Context packs: deterministic reference-closure bundles

## Problem

The archaeology's growing consumer is a context window. A session — or,
in the project vision, a contributor commissioned into an inherited
repository — needs "this artifact plus what it stands on": the target,
the artifacts its markers cite, transitively to some depth. Today that
bundle is hand-assembled with repeated `show` calls and file reads,
and every session assembles it differently. Relevance surfacing is the
project's stated holy grail; its mechanical precursor is much smaller
and needs no model: given a chosen artifact, produce its reference
closure deterministically.

## Sketch

`strata pack <ref>` (final name open) emits one deterministic stream:
the target artifact followed by its referenced artifacts to a bounded
depth, each delimited with its path and identity, in stable order.
Closure is defined purely by markers — no semantics, no ranking. The
output is a disposable projection under
[[dec-bootstrap-files-canonical|decision 1]]: nothing may come to
depend on a pack that the canonical files cannot answer.

Composes with [[ide_01KY7S6GK44ZVC8X2CF4KCM8MD|reverse reference
lookup]] — an optional flag could walk backlinks instead of forward
references — and gives [[idea-proposal-relevance-surfacing|relevance
surfacing]] a delivery vehicle: when ranking exists someday, it
reorders and prunes a pack; the container comes first.

## Boundaries

- Not semantic: no embeddings, no ranking beyond deterministic order.
- Never a cache: no other command may require a pack to exist.
- Explosion is handled honestly: if the closure exceeds the depth or
  size bound, the truncation is stated in the output, never silent.

## Evidence

The sprint 7 pitch reconstructed context by reading roughly fifteen
files whose selection was exactly a reference closure walked by hand.
Session-orientation and commissioning scenarios
([[ide_01KY7QF5FKX30PHTQ320MG4QXS|idea 21]]) both reduce to "bundle
what this artifact stands on" as their first step.

Proposed by Claude during the sprint 7 pitch, 2026-07-23.
