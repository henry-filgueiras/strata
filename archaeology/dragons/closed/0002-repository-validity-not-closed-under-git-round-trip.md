---
id: drg-bootstrap-git-round-trip
sequence: 2
kind: dragon
status: closed
created: 2026-07-20
resolved-by: "[[dec-bootstrap-repo-marker|repository marker and init safety]]"
---

# Repository validity is not closed under Git round-trip

## Context

`strata init` creates the required directories `archaeology/dragons/open`
and `archaeology/dragons/closed`. Git does not track empty directories, so
committing a freshly initialized repository and cloning it preserves
`.strata.toml` while silently dropping the empty layout.

Every read and write path then fails with `error[malformed-artifact]:
required dragon directory is missing`, although no canonical data was lost:
empty directories carry no artifact content. The failure is a false
corruption diagnosis of a state Git is documented to produce. Re-running
`strata init` recovers, but the diagnosis blames the user's repository for
Strata's own invariant.

## Question

Should repository validity be defined only over states Git can round-trip —
the marker alone defines the repository, with the directory layout
demand-materialized — instead of treating missing empty directories as
malformed state?

## Constraints

- The `.strata.toml` marker must remain explicit; no ordinary command may
  create a repository root implicitly.
- Reads must not mutate the repository, so tolerance on the read side means
  treating a missing managed directory as an empty collection, not creating
  it.
- A non-directory object occupying a managed path is a real conflict and
  must remain a typed error.
- `doctor` must not classify Git-inevitable states as corruption.

## Candidate direction

Define the repository by its valid marker alone. Readers treat a missing
managed directory as an empty collection; writers materialize missing
directories on demand under the discovered root, reusing the existing
conflict-checked directory creation. `init` remains a convergent
convenience that eagerly creates the skeleton. A `.gitkeep` convention was
considered and rejected as symptom-level: it only helps repositories
initialized after the change and re-imposes the problem on every future
managed directory.

## Resolution criteria

Decide and implement before task 0005 (`doctor`) is specified, so the
validator does not institutionalize the false alarm. Record the outcome as
an update to decision 0005.

## Resolution (2026-07-21)

Closed by adopting the candidate direction, recorded as the 2026-07-21
update to decision 0005 (`dec-bootstrap-repo-marker`): the marker alone
defines the repository, readers treat missing managed directories as
empty collections, writers materialize them on demand with the same
conflict checks `init` uses, and a non-directory at a managed path stays
a typed conflict. Verified by unit tests on the read and write paths and
an end-to-end test simulating the clone of a freshly initialized
repository.
