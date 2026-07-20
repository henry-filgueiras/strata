---
id: dec-bootstrap-stable-identity
sequence: 2
kind: decision
status: accepted
created: 2026-07-20
---

# Separate stable identity from display sequence

## Context

Sequential filename prefixes make artifacts easy to sort and reference:

```text
0007-wasm-stack-pressure.md
```

Two concurrent Git branches may independently allocate the same next number.
Renaming artifacts during merge should not change their durable identity.

## Decision

Artifacts will eventually carry both:

- a stable machine identity, likely a ULID;
- a collection-scoped display sequence used in filenames and human references.

The exact metadata schema remains provisional during bootstrap.

## Consequences

- Filename sequences may be repaired after branch collisions.
- Links and machine operations should prefer stable identities.
- Humans retain compact sortable filenames.
