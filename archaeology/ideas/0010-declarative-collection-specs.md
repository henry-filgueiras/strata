---
id: idea-declarative-collection-specs
sequence: 10
kind: idea
status: parked
created: 2026-07-21
---

# Declarative collection specs instead of a policy-template framework

## Problem

The dragon collection's mechanics — directory layout per lifecycle
state, front-matter schema, valid transitions, projection shapes — are
hardcoded. A second CLI-managed collection (ideas is the obvious next)
either copy-pastes that or generalizes it. The tempting generalization
is a C++-style compile-time policy framework: type-level knobs for
serialize/validate/deserialize, state machines, projection mappings,
invoked from thin per-collection product layers.

## Sketch

Generalize with data, not types. A `CollectionSpec` is a plain value —
name and plural, id prefix, lifecycle states with their directories and
allowed transitions, required front-matter fields, payload format,
projection field set — interpreted by one generic engine that already
knows how to scan, parse, validate, transition, and project. Behavior
escapes to a trait only where data cannot express it (payload codecs:
Markdown, JSON, JSONL — the seam decision 3 already reserved).

Why data over type-level machinery, opinionated:

- a spec that is a value can be printed, diffed, tested as a table, and
  read by `doctor` to validate *itself*; a template instantiation
  cannot;
- it keeps the door open to user-defined collections declared in
  `.strata.toml` — the natural end-state for a tool whose repositories
  must remain understandable without the executable. A compile-time
  tower forecloses that future; a spec interpreter gets it nearly free;
- Rust punishes the C++ pattern: trait-solver errors and monomorphization
  sprawl buy nothing here, because none of these knobs are
  performance-critical — the workload is parsing a handful of small
  files;
- rule of three: derive the spec shape from the second and third
  concrete collections, not ahead of them. The framework should be
  extracted from working duplicated code, or it will encode guesses.

The instinct itself is sound — collections *should* become declarations
consumed by shared machinery. The disagreement is only about when the
abstraction is earned and which axis (values, not types) carries it.

## Evidence

CLAUDE.md: bootstrap may hardcode one collection, "core abstractions
must not assume every artifact is Markdown", and speculative frameworks
are explicitly warned against — leave seams, extract later. Decision 3
(`dec-bootstrap-payload-separation`) is the reserved codec seam.
Sprint-2-era friction (ideas managed by hand while dragons are managed
by the tool) is the concrete forcing function. Prior art: serde's
derive-plus-attributes model (declarative surface, generic engine),
Kubernetes CRDs (collections as data), and the general "rule of three"
extraction discipline.
