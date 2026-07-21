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

## Update (2026-07-20): empty directories do not survive Git round-trip

Git does not track empty directories. Committing a freshly initialized
repository and cloning it preserves `.strata.toml` but drops the empty
`archaeology/dragons/open` and `archaeology/dragons/closed` directories;
`list` and `new` then fail with a `malformed-artifact` error claiming the
layout is damaged, though no canonical data was lost.

"Requires" therefore overstates what this decision can guarantee: the
layout requirement is not closed under `git clone`, and the resulting
error is a false corruption diagnosis of a state Git inevitably produces.
Re-running `strata init` remains a safe recovery, and the mutation-safety
contract above is unaffected.

Whether validity should instead be defined by the marker alone, with
managed directories materialized on demand, is tracked as dragon 0002
(`drg-bootstrap-git-round-trip`). Its resolution will be recorded here and
must land before `doctor` (task 0005) is specified.

## Update (2026-07-21): validity is defined by the marker alone

Dragon 0002 is resolved by adopting its candidate direction. Repository
validity is closed under Git round-trip:

- the valid `.strata.toml` marker alone defines a repository; the
  directory layout is a convenience, not a validity requirement;
- readers treat a missing managed directory as an empty collection and
  never mutate the repository to compensate;
- writers materialize missing managed directories on demand, reusing the
  conflict-checked, symlink-refusing directory creation `init` uses;
- a non-directory object occupying a managed path remains a typed
  `artifact-conflict` in every code path;
- `init` is unchanged: it still eagerly creates the skeleton and remains
  a safe, convergent convenience.

Consequently `doctor` must not report missing empty managed directories
as corruption — states Git inevitably produces are healthy. A `.gitkeep`
convention was considered and rejected as symptom-level (it cannot help
repositories initialized before the change and re-imposes the problem on
every future managed directory).
