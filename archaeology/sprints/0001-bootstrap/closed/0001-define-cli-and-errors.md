---
id: tsk-bootstrap-cli-errors
sequence: 1
kind: task
status: closed
sprint: spr-bootstrap
created: 2026-07-20
closed: 2026-07-20
---

# Define the bootstrap CLI and error model

## Objective

Define the command surface and typed errors for the initial vertical slice.

## Acceptance criteria

- `strata --help` clearly exposes bootstrap commands.
- Commands are represented with typed `clap` structures.
- Errors distinguish invalid invocation, missing repository, artifact conflict,
  malformed artifact, and filesystem failure.
- Automated callers are not required to parse error prose.
- No speculative daemon or networking abstractions are introduced.

## Result

Restructured the crate as a library (`src/cli.rs`, `src/error.rs`) plus a thin
binary so tests and later tasks share the types.

- `cli::Cli` / `cli::Command` define `init`, `new`, `list`, `show`, `doctor`
  with typed `clap` derive structures; `--help` lists all five.
- `cli::Collection` hardcodes `dragon` (accepting singular and plural) and
  `cli::ArtifactRef` parses `collection:sequence` references with sequences
  starting at 1.
- `error::Error` distinguishes the five required categories plus a
  transitional `unimplemented` category for stub commands; machines read the
  exit code and the leading `error[<code>]:` stderr token, never prose
  (recorded as decision 0004).
- All commands are stubs exiting 1 with `error[unimplemented]`; no artifact
  CRUD, daemon, or networking abstractions were added.
- `--json` was deliberately not added yet: placeholder flags are forbidden and
  it gains meaningful behavior with task 0004.

Verified with 23 tests (unit tests for reference parsing and the error
contract; integration tests driving the built binary), plus
`cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`,
all clean.
