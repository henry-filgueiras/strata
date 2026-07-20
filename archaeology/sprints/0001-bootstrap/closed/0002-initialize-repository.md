---
id: tsk-bootstrap-init
sequence: 2
kind: task
status: closed
sprint: spr-bootstrap
created: 2026-07-20
closed: 2026-07-20
---

# Initialize a Strata repository

## Objective

Implement `strata init` for the smallest supported repository layout.

## Acceptance criteria

- Creates required directories and configuration safely.
- Refuses to overwrite conflicting existing files.
- Re-running against an already valid repository is non-destructive.
- Works in a temporary non-Git directory.
- Partial failures do not leave truncated files.

## Result

Implemented in a new `repo` module (`src/repo.rs`); `main.rs` remains a thin
dispatch/rendering layer that resolves the current working directory and
prints what was created.

- Marker/config: `.strata.toml` containing `version = 1`; required
  directories `archaeology/dragons/open` and `archaeology/dragons/closed`.
  Contract recorded as decision 0005.
- An existing config is accepted only as a regular file with a supported
  config (unknown keys tolerated); invalid contents are `malformed-artifact`
  (exit 5), and a directory/symlink at the config path or a non-directory on
  a required directory path is `artifact-conflict` (exit 4). No new error
  categories or exit codes were added.
- The config is written last through an exclusive temporary file plus atomic
  no-clobber persist: `.strata.toml` is never partial or truncated after a
  failure. Directory creation is documented as non-transactional (a failed
  run may leave empty directories; re-run converges).
- No path arguments, upward root search, Git integration, or collection
  generalization were added.

## Verification

`scripts/check.sh` clean: `cargo fmt --check`, `cargo test` (40 tests:
26 unit, 14 integration), `cargo clippy --all-targets --all-features
-- -D warnings`. Coverage includes: fresh init in an empty non-Git temp
directory; expected config bytes and layout; non-destructive rerun;
byte-for-byte preservation of a hand-edited valid config; refusal of
unsupported versions, unparseable TOML, and a directory or symlink at the
config path; refusal of a file occupying a required directory path; a
naturally induced write failure (read-only root) leaving no partial config;
and binary-level CLI tests asserting exit codes and `error[<code>]:` tokens.
Also smoke-tested manually in a scratch directory.
