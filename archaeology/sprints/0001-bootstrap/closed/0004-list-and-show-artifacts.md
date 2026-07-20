---
id: tsk-bootstrap-list-show
sequence: 4
kind: task
status: closed
sprint: spr-bootstrap
created: 2026-07-20
closed: 2026-07-20
---

# List and show artifacts

## Objective

Rediscover artifacts from canonical files and expose human and machine
projections.

## Acceptance criteria

- `strata list dragons` produces concise human-readable output.
- `strata list dragons --json` produces deterministic structured output.
- `strata show` can resolve a stable identity or unambiguous human reference.
- Malformed files are reported rather than silently skipped.
- JSON field names are documented by tests.

## Result

Implemented in a new read-only module (`src/read.rs`): front-matter parsing,
managed-directory scanning, deterministic sorting, and reference resolution.
`main.rs` stays a thin dispatch/rendering layer; `doctor` is the only
remaining stub.

- Parsing: front matter is split at `---` delimiters and deserialized with a
  real mapping parser (`serde_yaml_ng`, the maintained successor to the
  archived `serde_yaml`) into a typed struct requiring `id`, `sequence`,
  `kind`, `status`, `created`; unknown fields are tolerated. Semantic checks
  enforce non-empty `id`/`created`, `kind: dragon`, `status` open/closed
  agreeing with `open/`/`closed/` placement, sequence in 1..=9999 and equal
  to the `NNNN-slug.md` filename sequence. The title is the single ATX
  `# Title` heading after front matter (fenced code blocks are skipped;
  setext headings are not recognized); missing or duplicate titles are
  malformed. IDs stay opaque — nothing assumes ULID structure, proven by a
  hand-seeded legacy-ID test artifact.
- Read model: one typed `Summary` (`id`, `sequence`, `kind`, `status`,
  `title`, `created`, repo-relative `path` with `/` separators) feeds both
  the human and `--json` projections; absolute paths never appear in output.
- Listing: sorts by sequence then repo-relative path; human mode prints
  `dragon:N  status  title  (path)` per artifact with a clear empty-state
  message; `--json` emits a deterministic snake_case array (exact shape
  pinned by tests, `[]` when empty). Duplicate sequences list fine — only
  resolution treats them as ambiguous.
- Showing: `strata show` accepts `dragon:N` or an opaque stable ID (any
  argument containing `:` parses as a human reference; anything else is an
  ID). Exactly one match succeeds; zero is `artifact-not-found` (exit 7);
  several is `ambiguous-reference` (exit 8) naming every candidate. Human
  output reproduces the canonical file byte-for-byte; `--json` adds
  `content` with the exact contents to the summary fields.
- Errors: the two new categories were appended to decision 0004 without
  renumbering; any malformed managed file is a typed `malformed-artifact`
  error naming its path — reads never silently skip. Scanning stops at the
  first problem; the repository-wide report remains `doctor`'s job.
- No mutation paths, lifecycle transitions, other collections, Git
  integration, or indexing were added.

## Verification

`scripts/check.sh` clean: `cargo fmt --check`, `cargo test` (116 tests: 76
unit, 40 integration across `cli`/`init`/`new`/`list_show`),
`cargo clippy --all-targets --all-features -- -D warnings`. Coverage
includes: list from root and nested directories; deterministic ordering
across open and closed including duplicate-sequence tiebreak by path; exact
JSON array bytes; empty-collection human and JSON modes; show by generated
ULID-style ID, legacy hand-seeded ID, and `dragon:N`; byte-for-byte human
show output and exact `content` in JSON show; not-found by ID and by
sequence (exit 7); duplicate sequence and duplicate ID ambiguity (exit 8);
malformed front matter, missing field, wrong type, wrong kind, invalid
status, status/placement mismatch, filename/front-matter sequence mismatch,
missing/duplicate/empty H1, fenced `#` lines not treated as titles,
malformed filenames, dot-entry exclusion; and binary-level assertions on
exit codes, `error[<code>]:` stderr tokens, and stdout cleanliness on both
success and failure. Also smoke-tested init → new → list/show (human and
JSON, by sequence and by ID) in a scratch directory.
