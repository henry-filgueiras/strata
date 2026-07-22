---
id: drg_01KY3C0S3JQKEMEB9BH6NVJ35F
sequence: 4
kind: dragon
status: open
created: 2026-07-21
---

# Power-loss durability of mutations

## Context

Every Strata mutation contract deliberately excludes power loss and
kernel crashes: the safe-write path stages content in a temporary file
and persists it with an atomic rename, but nothing is fsynced — not the
temporary's contents, not the destination's parent directory. The
`repo.rs` write contract has documented this scoping since sprint 1
("crash durability — fsync — is out of scope for bootstrap"); decision 8
(`dec-mutation-failure-classes`) now makes the exclusion an explicit
failure class rather than a code comment, after external review of the
sprint 2 transition contract surfaced it (thread
`cmt-transition-crash-contract`).

The risk is real, not theoretical. On ext4 with delayed allocation, a
rename-based replace whose data was never fsynced can surface as a
zero-length or truncated file after an unclean shutdown; the
`auto_da_alloc` heuristic narrows the window for rename-over-existing
but is not a contract, and does not cover no-clobber creates. On APFS,
`fsync` alone does not force a barrier through the drive cache;
`F_FULLFSYNC` is required for a real guarantee. Durable directory
entries additionally require fsyncing the parent directory. So an
artifact created or transitioned shortly before power failure can be
lost or torn even though every in-process guarantee held.

## Question

When, if ever, does Strata need real crash durability on its safe-write
path — fsync of staged content plus parent-directory fsync, with
`F_FULLFSYNC` on macOS — and is it unconditional, opt-in, or permanently
out of scope with documented recovery guidance instead?

## Constraints

- Files are canonical (decision 1): no journal, WAL, or recovery log may
  become state a reader needs to understand the repository.
- Git already provides recovery for committed artifacts; the exposure is
  limited to mutations made since the last commit.
- Durability syscalls cost latency on every mutation; the price must be
  justified by evidence of real loss, not hypotheticals.
- `doctor` should be able to describe torn states (for example a
  zero-length managed `.md` file) helpfully whatever is decided.

## Resolution criteria

A recorded decision either adds durability to the safe-write path
(scope, platforms, and measured cost documented) or permanently accepts
the exclusion with recovery guidance (recreate, or restore from Git) —
and `doctor`'s treatment of torn files is specified either way.
