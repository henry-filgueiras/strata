---
id: tsk_01KY6364E105F7AWT7RAZ264WZ
sequence: 26
kind: task
status: pending
sprint: spr_01KY61D615FAC8VVSTD7QXX1DW
created: 2026-07-22
---

# Adopt a deliberate line-ending policy

## Objective

Close case D of comment thread 6. Front-matter discovery requires
literal LF delimiters (`---\n`, `\n---\n`), the repository ships no
`.gitattributes`, and CI runs only on Ubuntu. A Windows-default
checkout (`core.autocrlf=true`) rewrites every artifact to CRLF, after
which every artifact fails to parse — reproduced: one such checkout
turns the whole corpus into `malformed-artifact: missing front
matter`, a diagnosis that misdescribes the actual state and gives the
user no repair path. "Git-friendly" currently means "Git-friendly on
LF platforms only", and nothing decided that.

Closure property from the thread: supported checkout line endings are
parsed and byte-preserved deliberately (5).

## Acceptance criteria

- A recorded decision states the line-ending posture: either artifacts
  are LF-only and the repository enforces that at the Git boundary, or
  the parser accepts CRLF and every mutation byte-preserves it. The
  choice is recorded with its tradeoffs (byte-exact splicing, safe
  writes, and `content_is_preserved_byte_for_byte` semantics all touch
  it).
- A `.gitattributes` implementing the chosen posture for `*.md` and
  `.strata.toml` ships at the repository root, and `strata init` is
  considered as a place to materialize the same policy for new
  repositories (adopt or explicitly decline in the decision).
- When Strata encounters an artifact whose line endings the posture
  excludes, the diagnosis names line endings as the cause and the
  repair (not "missing front matter").
- Regression tests cover a CRLF artifact for whichever posture was
  chosen: parsed-and-preserved, or refused with the line-ending
  diagnosis; plus a doctor run over a CRLF corpus.
- `scripts/check.sh` and `strata doctor` are green at close.

## Result
