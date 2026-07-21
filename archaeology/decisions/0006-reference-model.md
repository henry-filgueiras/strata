---
id: dec-bootstrap-reference-model
sequence: 6
kind: decision
status: accepted
created: 2026-07-20
---

# Cross-references target stable identities and bind at authorship

## Context

Artifacts increasingly refer to one another (dragons to decisions, decision
updates to dragons). Today those references are untyped prose in
inconsistent forms — sometimes a display sequence ("dragon 0002"),
sometimes a stable ID (`drg-bootstrap-git-round-trip`), sometimes both.
Nothing defines what a reference points at, when it is resolved, or what
tooling may do with it.

Display sequences are known to collide across branches (dragon 0001), so
any reference model built on them inherits that collision. Decision 0002
already commits machine operations to stable identities.

## Decision

The principles below are settled ahead of implementation. No bootstrap task
implements them; they constrain future syntax and tooling choices.

- References target stable artifact IDs, never display sequences. A
  human-oriented label may accompany the ID inside the reference; the label
  is frozen non-authoritative decoration, the ID is the truth.
- Binding — resolving human sugar such as `dragon:2` to a stable ID — is a
  write-time act performed in the author's working tree, where the sugar
  has a well-defined referent. Automation (CI, hooks) may verify that
  references are bound and block when they are not, but must never perform
  binding or rewrite references itself: late binding can silently resolve
  to an artifact the author did not mean.
- Canonical files store outgoing references only. Backlinks, the full
  reference graph, and any analysis over it (dangling detection, cycles)
  are derived projections, rebuildable from the corpus, per decision 0001.
- References are always explicit markers. Typed, machine-actionable edges
  (for example supersedes, resolves, amends) live as front-matter fields;
  untyped associations live as explicit inline markers in prose. Nothing is
  ever inferred from unmarked natural language: prose legitimately contains
  negated, quoted, and hypothetical reference-shaped text.
- Cycle or consistency checking is only meaningful per typed edge kind;
  untyped association cycles are normal, not defects.
- An unbound sugar reference in a canonical file is legal but weak: a
  repairable diagnostic, never corruption. Tooling upgrades it via an
  explicit bind operation; writers without tooling are not blocked.
- The sugar form must be a relaxation of the canonical grammar — one
  grammar, two strictness levels — so that binding is a repair within one
  language, not a translation between two.
- Canonical labels are frozen at bind time and are not refreshed by
  automation. Read-side projections may display current truth (and note
  drift); batch label rewriting is permitted only inside an explicit repair
  operation (such as the future sequence-collision repair from dragon
  0001), through Strata's safe-write path, as one reviewable diff.

## Open points

- The concrete marker syntax (wikilink-style versus Markdown-link-style,
  exact front-matter key names) is deliberately undecided. Acceptance test
  for any candidate: it must read acceptably in a raw GitHub PR diff with
  no tooling present.
- Which typed edge kinds exist, and their doctor semantics (a dangling
  typed edge is likely real corruption; a dangling untyped marker is not),
  are deferred until a consumer exists.

Both open points are promoted to dragon 3
(`drg_01KY169X7W0YXJ5QFV4D1MK4FB`), which owns their resolution; the
resolving decision should be recorded as an update here or as a successor
decision.

## Consequences

- Writing references correctly requires no tooling; writing them durably
  requires one bind step. Ergonomics degrade gracefully instead of gating
  on environment.
- References survive branch merges and sequence collisions untouched,
  because they never depended on sequences after binding.
- Until the marker syntax is chosen, cross-references remain prose naming
  both forms (sequence for humans, stable ID for machines), as this file
  does — the interim convention this decision itself must use.
