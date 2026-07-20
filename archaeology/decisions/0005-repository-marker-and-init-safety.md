---
id: dec-bootstrap-repo-marker
sequence: 5
kind: decision
status: accepted
created: 2026-07-20
---

# Repository marker, config versioning, and init safety

## Context

`strata init` needed a durable definition of what marks a directory as a
Strata repository, how the config evolves, and what mutation guarantees
initialization makes.

## Decision

- The repository root is marked by `.strata.toml`, a regular file (symlinks
  are rejected) containing a TOML table with an integer `version` key.
- `version = 1` is the current schema. Unknown keys are tolerated within a
  supported version, so additive keys never make older configs unreadable;
  an unsupported version is a `malformed-artifact` error, never overwritten.
- The bootstrap layout requires `archaeology/dragons/open` and
  `archaeology/dragons/closed`.
- Init mutation-safety semantics:
  - existing files are never modified, truncated, or replaced;
  - the config is written last via an exclusive temporary file and an
    atomic no-clobber persist, so `.strata.toml` is all-or-nothing at
    process level (fsync durability is out of bootstrap scope);
  - directory creation is not transactional: a failed run may leave newly
    created empty directories, which are harmless and converge on re-run;
  - any non-directory object on a required directory path, or non-regular
    object at the config path, is an `artifact-conflict` error.

## Consequences

- Tools can detect a Strata repository by the presence of a valid
  `.strata.toml` without executing Strata.
- Future schema changes bump `version`; readers reject versions they do not
  support instead of guessing.
- Re-running `strata init` is always safe.
- The existing error categories from [decision 0004] cover config problems
  (`malformed-artifact`) and occupied paths (`artifact-conflict`); no new
  categories or exit codes were introduced.
