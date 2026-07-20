---
id: dec-bootstrap-payload-separation
sequence: 3
kind: decision
status: accepted
created: 2026-07-20
---

# Payload format is not the artifact model

## Context

Narrative decisions and investigations fit Markdown well.

Other repository evidence is naturally represented as structured JSON or
row-oriented JSONL, including benchmark runs, performance samples, traces, and
experiment observations.

Forcing every artifact into semi-structured Markdown would weaken machine
readability and encourage duplicated sources of truth.

## Decision

Artifact identity, collection, lifecycle, and placement are independent of
payload representation.

Markdown is the first supported codec, not the universal storage format.

JSON and JSONL support are intentionally deferred until the Markdown workflow
is proven.

## Consequences

- Core abstractions must not embed Markdown-specific assumptions unnecessarily.
- Machine-friendly evidence can remain canonical.
- Human-readable tables and charts may later be generated projections.
