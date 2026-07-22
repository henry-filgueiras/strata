---
id: idea-strata-edit
sequence: 3
kind: idea
status: parked
created: 2026-07-20
---

# `strata edit`: validated `$EDITOR` round-trip

## Problem

The create-then-hand-edit workflow leaves canonical files unvalidated
until some later read explodes: `strata new` writes a correct skeleton,
then every subsequent edit bypasses the parser entirely, and managed
front-matter fields (`id`, `sequence`) sit exposed to accidental
corruption. This friction was felt concretely while filling in dragon 3
minutes after creating it.

## Sketch

`strata edit <reference>` opens a projection in `$EDITOR` and validates on
return, in the `git rebase -i` / `kubectl edit` style: parse with the
task-0004 parser, bind references, refuse malformed results with a
retry/abandon loop, and persist through the safe-write path. Constraints
fixed by decision 0007: the projection is a relaxation of the canonical
grammar, untouched content round-trips byte-identical, managed fields
appear as protected scaffolding, and an aborted or crashed session never
loses the user's text. Also the natural editing story for future JSON and
JSONL payloads.

## Evidence

Decision 0007 (`dec-bootstrap-interaction-surfaces`), which pre-approves
the pattern and its constraints; observed friction creating dragon 3;
prior art in `visudo`, `crontab -e`, `kubectl edit`.
