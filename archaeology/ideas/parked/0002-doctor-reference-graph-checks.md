---
id: idea-doctor-reference-graph
sequence: 2
kind: idea
status: parked
created: 2026-07-20
---

# Doctor checks over the derived reference graph

## Problem

Cross-references and the promises they carry are invisible to tooling: a
dragon can pledge "record the outcome in decision 0005" and nothing will
ever notice the pledge dangling. Duplicate-collision detection exists for
sequences (dragon 1) but nothing analogous exists for references.

## Sketch

Doctor derives the full reference graph (typed front-matter edges plus
untyped inline markers, tagged by provenance) as a disposable projection,
then reports per decision 0006 severity: a dangling typed edge is a
corruption-level finding; unbound sugar and dangling untyped markers are
diagnostics; frozen-label drift is information only. Cycle checks apply
only to typed edge kinds with DAG semantics.

## Evidence

Decision 0006 (`dec-bootstrap-reference-model`), which requires each typed
edge kind to define doctor semantics up front; the stringly-typed
cross-reference gap observed while recording dragon 2. Blocked on dragon 3
(`drg_01KY169X7W0YXJ5QFV4D1MK4FB`).
