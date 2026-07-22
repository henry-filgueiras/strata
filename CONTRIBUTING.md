# Contributing to Strata

Thanks for your interest. Strata is early — a single maintainer proving a
small vertical slice — so the most valuable contributions right now are bug
reports, ideas, and focused fixes rather than large features.

This repository is unusual in one way that affects every contribution: it is
its own first user. The project's decisions, known risks, and work items live
in [`archaeology/`](archaeology/) as ordinary files, and substantive changes
are expected to keep that record current. It is also, deliberately, a case
study in human–AI collaboration; the same workflow applies whether a change
is written by a person, an agent, or both.

## Building and testing

Stable Rust toolchain, then:

```sh
cargo build
cargo run -- --help
scripts/check.sh   # rustfmt --check, full test suite, clippy -D warnings
```

`scripts/check.sh` must pass before any pull request. Filesystem mutation is
the primary correctness surface; tests use temporary directories and should
never touch the working repository.

## Before you write code

1. Read [`CLAUDE.md`](CLAUDE.md) — it holds the project invariants (files are
   canonical, no hostage-taking, identity vs. display sequence, and so on).
   These are enforced in review.
2. Read the current sprint under `archaeology/sprints/` and its pending
   tasks. Work that belongs to no sprint and serves no recorded idea or
   dragon is likely to be declined regardless of quality — open an issue
   first.
3. Check open dragons (`archaeology/dragons/open/`) and recorded decisions so
   you don't reopen a settled tradeoff without new evidence.

## Making changes

- Prefer small, reviewable vertical slices; don't combine filesystem CRUD
  with unrelated concerns.
- Match the existing code style; `scripts/check.sh` enforces formatting and
  lints.
- Update the archaeology alongside the change: close or amend the relevant
  task, record unresolved risks as dragons, record consequential
  architectural choices as decisions. Routine changes don't need ceremony —
  see "What deserves durable archaeology" in `CLAUDE.md`.
- Commit messages follow `area: what changed` — lowercase, imperative
  (`doctor: validate repository invariants`, `ideas: park strata fortune`).

## Bugs and ideas

- **Bug reports**: use the bug-report issue template. Include the command
  run, the repository state, and `strata doctor` output where relevant.
- **Ideas**: use the idea issue template, shaped like the project's idea
  artifacts (Problem / Sketch / Evidence). Ideas are never load-bearing: an
  accepted proposal becomes a parked idea artifact in the archaeology, not a
  roadmap promise.
- **Security issues**: do not open a public issue — see
  [`SECURITY.md`](SECURITY.md).

## Licensing

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed under MIT OR Apache-2.0, without any
additional terms or conditions.

## Conduct

Participation is governed by the
[Code of Conduct](CODE_OF_CONDUCT.md).
