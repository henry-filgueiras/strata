---
id: dec-bootstrap-files-canonical
sequence: 1
kind: decision
status: accepted
created: 2026-07-20
---

# Files are canonical

## Context

Strata needs durable project memory that humans, Git, shell tools, and coding
agents can all inspect.

A database could simplify querying, but it would make repository understanding
depend on Strata-specific storage and tooling.

## Decision

Canonical project records are ordinary repository files.

Databases, indexes, generated summaries, dashboards, and embeddings may exist
only as disposable projections that can be rebuilt from canonical files.

## Consequences

- Git history, review, branching, blame, and revert work naturally.
- Repositories remain legible without Strata.
- File layout and mutation safety become important correctness surfaces.
- Query acceleration may later require a rebuildable index.
