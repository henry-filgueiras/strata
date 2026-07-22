---
id: drg-bootstrap-branch-collisions
sequence: 1
kind: dragon
status: open
created: 2026-07-20
---

# Branch sequence collisions

## Context

Two branches can independently inspect the same collection, choose the same next
display sequence, and create different artifacts with identical numeric
prefixes.

## Question

What repair policy should Strata use when duplicate display sequences are found
after branches merge?

## Constraints

- Stable artifact identities must not change.
- Existing artifacts should not be renumbered casually.
- Repair must never overwrite or lose content.
- `doctor` should detect collisions deterministically.

## Candidate direction

Treat sequence numbers as repairable presentation metadata. Provide an explicit
future repair operation rather than silently renumbering during ordinary reads.

## Resolution criteria

Resolve after the bootstrap scanner and validator make the collision behavior
concrete enough to test.
