---
id: tsk-bootstrap-create-dragon
sequence: 3
kind: task
status: closed
sprint: spr-bootstrap
created: 2026-07-20
closed: 2026-07-20
---

# Create a numbered dragon artifact

## Objective

Implement:

```sh
strata new dragon "Branch sequence collisions"
```

## Acceptance criteria

- Locates the repository root.
- Scans existing dragon display sequences.
- Allocates the next sequence safely.
- Generates a deterministic lowercase kebab-case filename.
- Assigns a stable identity.
- Writes valid Markdown front matter and template sections.
- Never overwrites an existing artifact.
- Failed creation does not leave a partial destination file.

## Result

Implemented in `src/artifact.rs` (creation, slugging, sequence allocation,
safe writes) plus `repo::discover` (upward root search); `main.rs` stays a
thin dispatch layer printing the reference and root-relative path.

- Discovery walks upward from the working directory to the first valid
  `.strata.toml`; a malformed or non-regular marker stops the search with a
  typed error instead of resolving to an outer repository.
- Generated IDs are `drg_` + uppercase ULID (policy recorded as an update to
  decision 0002); seeded non-ULID IDs remain valid and untouched.
- Sequences are `max(open ∪ closed) + 1`; malformed filenames in managed
  directories are typed errors naming the path (dot-entries are ignored);
  exhaustion beyond 9999 is a typed error.
- Slugs are lowercase ASCII kebab-case with collapsed separators; non-ASCII
  characters act as separators (no transliteration); unsluggable titles are
  rejected.
- Writes stage through a dot-prefixed temporary in the destination directory
  and persist with an atomic no-clobber rename: no overwrite, no partial
  destination. Concurrent allocation is deliberately not linearized; the
  boundary is documented in the `artifact` module docs, and duplicate
  sequences remain detectable by `doctor` (dragon 0001).
- Corrected `status: done` → `status: closed` in tasks 0001–0002 and added
  the placement/lifecycle invariant to `CLAUDE.md`.

## Verification

`scripts/check.sh` clean: `cargo fmt --check`, `cargo test` (68 tests: 47
unit, 21 integration across `cli`/`init`/`new`), `cargo clippy --all-targets
--all-features -- -D warnings`. Coverage includes creation from the root and
a nested directory, missing repository, malformed marker mid-walk, sequence
allocation across open and closed, zero-padded filenames, slug determinism
and non-ASCII behavior, unsluggable-title rejection, prefixed-ULID shape and
uniqueness, front matter and required headings, destination conflict without
overwrite or temporary litter, induced write failure leaving nothing behind,
sequence exhaustion, seeded ID preservation, and binary-level human output.
Also smoke-tested init + two creations in a scratch directory.
