# CLAUDE.md

## Project

Strata is a Git-friendly command-line tool for maintaining structured project archaeology and repository-local execution memory.

It manages human-readable files such as decisions, investigations, unresolved technical risks (“dragons”), development logs, sprints, and work items. These artifacts remain ordinary files that humans, shell tools, Git, and coding agents can inspect without Strata.

Strata may later provide rebuildable indexes, semantic retrieval, an MCP adapter, and other projections. Those are not part of the initial implementation.

## Core invariants

### Files are canonical

The filesystem is the source of truth.

Do not introduce a canonical database, hidden state store, remote service, or cache that is required to understand or modify a Strata repository.

Any future database or index must be disposable and completely rebuildable from canonical files and Git history.

### The tool must not hold repositories hostage

A repository managed by Strata must remain understandable and editable when the Strata executable is unavailable.

Artifacts should use broadly supported formats:

* Markdown for narrative documents
* JSON for structured documents
* JSONL/NDJSON for appendable or large row-oriented datasets

Initial implementation may support only a subset of these formats, but core abstractions must not assume that every artifact is Markdown.

### Intent-level operations enforce mechanics

Users and agents should express operations such as:

* create an artifact
* list artifacts
* inspect an artifact
* transition an artifact between states
* validate repository invariants

Strata is responsible for mechanics such as:

* choosing display sequence numbers
* generating safe filename slugs
* assigning stable identities
* selecting the correct directory
* updating metadata
* preventing destructive collisions
* reporting invalid repository state

### Human and machine interfaces share one core

Commands must provide clear human-readable output by default and deterministic machine-readable output through `--json`.

Do not implement separate semantics for interactive and automated clients.

### Git awareness is optional at the core

Strata should work in an ordinary filesystem directory.

Git integration may add history, change awareness, staging, or provenance, but core artifact operations must not require Git unless a feature specifically depends on Git.

### Derived output is not canonical

Generated indexes, Markdown tables, dashboards, summaries, visualizations, caches, and embeddings are projections.

They must not silently become the only location containing important project facts.

### Semantic systems advise; they do not define truth

Future embeddings or model-derived observations may suggest relationships or suspicious states.

They must not be treated as structural validation errors or mutate canonical artifacts without an explicit operation.

## Initial scope

The first usable release should implement a small filesystem protocol.

Target commands:

```text
strata init
strata new
strata list
strata show
strata move
strata doctor
```

The first implementation should focus on Markdown narrative artifacts.

JSON and JSONL support should be represented in the architecture but need not receive complete CRUD support until the Markdown workflow is stable.

Every command intended for automation should eventually support:

```text
--json
--quiet
--dry-run
--no-interactive
```

Only implement flags that have meaningful behavior. Do not add placeholder flags.

## Explicit non-goals for the bootstrap phase

Do not implement the following without a recorded decision and concrete evidence that the preceding layer is useful:

* background daemon
* filesystem watcher
* SQLite index
* embeddings
* semantic search
* GraphQL
* MCP server
* TUI
* web dashboard
* GitHub Issues synchronization
* automatic commits
* autonomous task selection
* multi-agent locking
* scheduling
* burndown charts
* generalized knowledge-graph ontology

These are possible future projections, not bootstrap requirements.

## Proposed artifact model

An artifact has identity and repository placement independent of payload representation.

Conceptually:

```rust
struct Artifact {
    id: ArtifactId,
    sequence: u64,
    collection: CollectionName,
    container: Option<ArtifactId>,
    state: Option<StateName>,
    title: String,
    payload_kind: PayloadKind,
    path: PathBuf,
}
```

Possible payload kinds:

```rust
enum PayloadKind {
    Markdown,
    Json,
    JsonLines,
    External,
}
```

Do not preserve this exact API if implementation experience reveals a simpler model. Preserve the separation between artifact metadata and payload representation.

### Stable IDs and display sequences

Sequential filename prefixes exist for sorting and convenient human reference:

```text
0007-wasm-stack-pressure.md
```

They are not durable identity because concurrent branches may allocate the same next number.

Artifacts should eventually carry a stable identifier, likely a ULID, while retaining the sequence number as presentation metadata.

Example:

```yaml
---
id: drg_01JZ8KX4S9A2M7QH
sequence: 7
kind: dragon
state: open
created: 2026-07-20
---
```

The exact metadata format is not final.

## Repository conventions

Until Strata can manage its own archaeology, this repository uses the following manually maintained structure:

```text
archaeology/
├── decisions/
├── dragons/
│   ├── open/
│   └── closed/
├── logs/
└── sprints/
    └── NNNN-name/
        ├── sprint.md
        ├── pending/
        └── closed/
```

Use four-digit, zero-padded display sequences.

Use lowercase kebab-case filenames.

Never reuse a sequence number within the same numbering scope merely because an artifact was deleted or moved.

Prefer moving artifacts to a terminal state over deleting historical records.

Do not renumber existing artifacts for cosmetic reasons.

## Archaeology responsibilities

Before substantial work:

1. Read this file.
2. Read the current sprint document.
3. Inspect pending work items.
4. Inspect relevant open dragons and decisions.
5. Confirm that the intended change belongs to the current sprint or record why it does not.

During work:

1. Preserve repository history rather than rewriting prior conclusions as though they never existed.
2. Record durable discoveries that would otherwise force future contributors to repeat investigation.
3. Create or update a dragon when an unresolved technical risk materially affects the work.
4. Create a decision record when choosing between meaningful alternatives with future architectural consequences.
5. Do not log routine command execution or transient reasoning.

Before finishing:

1. Run formatting and tests.
2. Update the relevant work item with the result.
3. Move completed work from `pending` to `closed`.
4. Record unresolved findings as dragons.
5. Record durable architectural conclusions as decisions.
6. Verify that paths, metadata, and references remain consistent.
7. Summarize what changed, what was verified, and what remains open.

## What deserves durable archaeology

Record information whose absence would likely cause a future contributor to:

* repeat meaningful investigation
* violate an architectural invariant
* misunderstand why an unusual design exists
* reopen a settled tradeoff without new evidence
* overlook a known unresolved risk
* misinterpret generated output as canonical source data

Do not record:

* every command executed
* obvious implementation details already clear from code
* speculative future features with no decision or action attached
* verbose session transcripts
* temporary scratch reasoning
* celebratory summaries without durable technical content

## Change discipline

Prefer small, reviewable increments.

Do not combine core filesystem CRUD with indexing, semantic retrieval, networking, or agent protocol work.

When an attractive extension appears:

1. record it as a future idea or dragon if necessary;
2. identify the concrete bootstrap use case it would improve;
3. continue the current scoped task unless the extension blocks correctness.

Avoid designing abstractions solely for imagined future clients. Leave clean seams where inexpensive, but allow real usage to shape the APIs.

## Rust guidance

Use stable Rust.

Initial likely dependencies:

* `clap` for CLI parsing
* `serde` for structured data
* `toml` for configuration
* `serde_json` for JSON output
* `thiserror` for typed errors
* `anyhow` or `miette` at the application boundary
* `tempfile` for filesystem tests
* a small slugification crate if its behavior is deterministic and well tested
* a ULID crate when stable IDs are implemented

Avoid adding `git2` during bootstrap. Invoke the installed `git` command for narrowly scoped Git-aware features unless there is demonstrated need for libgit2.

Prefer standard-library filesystem operations over framework-heavy abstractions.

## Testing priorities

Filesystem mutation is the principal correctness surface.

Use temporary repositories/directories to test:

* initialization
* sequence allocation
* deterministic slug generation
* safe artifact creation
* refusal to overwrite existing files
* state transitions
* malformed metadata
* duplicate stable IDs
* duplicate sequence numbers
* configuration errors
* deterministic JSON output
* recovery from partial or invalid repository state

Tests must verify that failed operations do not lose or truncate artifact content.

When practical, mutations should write to a temporary file and atomically rename it into place.

## Output and error behavior

Human-readable errors should explain:

* what operation failed
* which path or artifact was involved
* what invariant was violated
* what the user can do next

Machine-readable output should use stable field names.

Do not require automated callers to parse prose.

Exit codes should eventually distinguish at least:

* success
* invalid invocation
* artifact not found
* ambiguous reference
* invalid transition
* repository invariant violation
* filesystem or edit conflict

The exact numeric assignments should be documented before being considered stable.

## Current bootstrap objective

Prove the smallest useful vertical slice:

1. initialize a repository configuration;
2. define one Markdown collection with states;
3. create a sequentially named artifact safely;
4. list discovered artifacts in human and JSON forms;
5. validate basic filename and metadata invariants;
6. exercise the workflow in Strata’s own repository.

Do not begin daemon, index, embedding, MCP, or GraphQL work until this workflow has been used successfully in real project sessions.
