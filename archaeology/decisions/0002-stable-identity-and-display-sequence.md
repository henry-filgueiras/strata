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

## Update (2026-07-20): bootstrap identity policy

Task 0003 settled the provisional schema for generated identities:

- `id` is a single opaque stable string; there is no second identity field.
- Strata-generated dragon IDs are `drg_` followed by a 26-character uppercase
  Crockford base32 ULID, e.g. `drg_01K0P6W5PK8T19H7M2V8W6YQ4C`.
- Hand-seeded IDs predating generation (e.g.
  `drg-bootstrap-branch-collisions`) remain valid forever; no reader may
  require an `id` to be a ULID, and existing IDs are never rewritten.
- Display sequences are allocated as `max(open ∪ closed) + 1` within the
  four-digit space; exhaustion beyond 9999 is a typed error, and duplicate
  sequences from concurrent allocation are left for `doctor` to detect
  (see dragon 0001).
