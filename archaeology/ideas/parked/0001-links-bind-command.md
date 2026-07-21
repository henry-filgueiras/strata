---
id: idea-links-bind-command
sequence: 1
kind: idea
status: parked
created: 2026-07-20
---

# `strata links bind` and its check mode

## Problem

Decision 0006 (`dec-bootstrap-reference-model`) makes unbound sugar
references legal but weak, repairable only by an explicit bind operation —
which does not exist. Automation is permitted to verify binding but never
perform it, so a blocking check mode is the only CI-compatible surface.

## Sketch

`strata links bind` resolves unbound references (`dragon:2` forms) to
their stable-ID canonical form in place, through the existing safe-write
path, as one reviewable diff. `strata links bind --check` mutates nothing,
exits nonzero listing every unbound reference, and slots into
`scripts/check.sh` and CI exactly like `cargo fmt --check`.

## Evidence

Decision 0006 (binding semantics); decision 0007
(`dec-bootstrap-interaction-surfaces`, no tooling rewrites prose outside
explicit operations); log 0002's rejected alternative of CI-performed
binding. Blocked on dragon 3 (`drg_01KY169X7W0YXJ5QFV4D1MK4FB`) choosing
the marker syntax.
