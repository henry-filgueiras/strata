# CLAUDE.md

## Project

Strata is a Git-friendly command-line tool for maintaining structured project
archaeology and repository-local execution memory.

It manages ordinary files representing decisions, unresolved technical risks
(“dragons”), investigations, development logs, sprints, work items, and
structured evidence.

Strata may eventually expose rebuildable indexes, semantic retrieval, an MCP
adapter, or other projections. Those are not part of the bootstrap phase.

## Core invariants

### Files are canonical

The filesystem is the source of truth.

Do not introduce a canonical database, hidden state store, remote service, or
cache required to understand or modify a Strata repository.

Any future index must be disposable and rebuildable from canonical files and,
where relevant, Git history.

### Strata must not hold repositories hostage

A Strata repository must remain understandable and editable without the Strata
executable.

Expected payload formats include:

- Markdown for narrative artifacts
- JSON for structured documents
- JSONL or NDJSON for row-oriented datasets

The bootstrap implementation may support only Markdown, but core abstractions
must not assume every artifact is Markdown.

### Artifact identity is separate from payload format

An artifact has identity, collection membership, lifecycle state, and repository
placement independently of whether its payload is Markdown, JSON, JSONL, or
another supported representation.

### Display sequences are not durable identity

Zero-padded filename prefixes exist for sorting and convenient human reference:

```text
0007-wasm-stack-pressure.md
```

Concurrent branches may allocate the same next sequence. Artifacts therefore
need a stable identity independent of their display sequence. A ULID is the
current likely choice.

### Intent-level commands enforce mechanics

Users and agents should express operations such as:

- create an artifact
- list artifacts
- inspect an artifact
- transition an artifact between states
- validate repository invariants

Strata handles numbering, slugging, identity allocation, path selection,
metadata consistency, collision prevention, and safe writes.

### Human and machine interfaces share one core

Human-readable output is the default.

Automation receives deterministic structured output through `--json`.

Do not implement separate semantics for human and automated callers.

### Git is optional at the core

Core filesystem operations should work outside a Git repository.

Git-aware features may provide provenance, history, staging, or change
detection, but Git should not be required unless a particular operation depends
on it.

### Derived projections are not canonical

Indexes, generated Markdown tables, dashboards, summaries, embeddings, and
visualizations are disposable projections.

They must not become the only location containing important project facts.

### Semantic systems advise; they do not define truth

Future embedding or model-derived observations may suggest relationships or
suspicious states.

They must not be treated as structural validation failures or mutate canonical
artifacts without an explicit operation.

## Bootstrap scope

Build the smallest useful vertical slice.

Initial target commands:

```text
strata init
strata new
strata list
strata show
strata doctor
```

The first implementation may hardcode a single `dragon` collection while the
behavior is proven.

Expected bootstrap workflow:

```sh
strata init
strata new dragon "Branch sequence collisions"
strata list dragons
strata list dragons --json
strata show dragon:1
strata doctor
```

The first implementation should support Markdown narrative artifacts.

JSON and JSONL should be acknowledged in the architecture but do not need full
CRUD support until the Markdown workflow is useful.

## Explicit non-goals

Do not implement these during bootstrap without a new recorded decision and
evidence that the preceding layer is useful:

- background daemon
- filesystem watcher
- SQLite index
- embeddings
- semantic search
- GraphQL
- MCP server
- TUI
- web dashboard
- GitHub Issues synchronization
- automatic commits
- autonomous task selection
- multi-agent locking
- generalized knowledge graph
- cross-repository indexing

Leave inexpensive seams where appropriate, but do not design speculative
frameworks for these features.

## Manual archaeology layout

Until Strata can manage its own records, maintain:

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

Conventions:

- use four-digit zero-padded display sequences;
- use lowercase kebab-case filenames;
- do not reuse deleted or moved sequence numbers;
- do not renumber existing artifacts cosmetically;
- prefer moving records to terminal states over deleting history;
- keep generated artifacts clearly distinguishable from canonical sources.

## Archaeology workflow

Before substantial work:

1. Read this file.
2. Read the current sprint.
3. Inspect pending work.
4. Inspect relevant decisions and open dragons.
5. Confirm the intended change belongs to the current sprint.

During work:

1. Preserve historical conclusions rather than rewriting history.
2. Record durable discoveries that would otherwise require repeated research.
3. Create or update a dragon for unresolved risks that materially affect work.
4. Create a decision record for consequential architectural choices.
5. Do not record routine commands, transient reasoning, or session transcripts.

Before finishing:

1. Run formatting, linting, and tests.
2. Update the relevant work item with its result.
3. Move completed work from `pending` to `closed`.
4. Record unresolved findings as dragons.
5. Record durable architectural conclusions as decisions.
6. Check that paths, metadata, and references remain consistent.
7. Summarize changes, verification, and remaining work.

## What deserves durable archaeology

Record information whose absence would likely cause a future contributor to:

- repeat meaningful investigation;
- violate an architectural invariant;
- misunderstand why an unusual design exists;
- reopen a settled tradeoff without new evidence;
- overlook a known unresolved risk;
- mistake generated output for canonical source data.

Do not record:

- every command executed;
- obvious implementation details already clear from code;
- speculative features with no decision or action attached;
- verbose transcripts;
- temporary scratch reasoning;
- ceremonial progress reports.

## Change discipline

Prefer small, reviewable vertical slices.

Do not combine filesystem CRUD with indexing, semantic retrieval, networking,
or agent protocol work.

When an attractive extension appears:

1. record it as a future idea when useful;
2. identify the concrete problem it would solve;
3. continue the scoped task unless the extension blocks correctness.

## Rust guidance

Use stable Rust.

Reasonable bootstrap dependencies include:

- `clap` for CLI parsing
- `serde` and `serde_json` for structured output
- `toml` for repository configuration
- `thiserror` for typed errors
- `anyhow` or `miette` at the application boundary
- `tempfile` for filesystem tests
- a deterministic slugification crate
- a ULID crate when stable identities are added

Avoid `git2` during bootstrap. Use the installed `git` command for narrowly
scoped Git-aware features unless demonstrated requirements justify libgit2.

Prefer standard-library filesystem operations and small focused abstractions.

## Testing priorities

Filesystem mutation is the primary correctness surface.

Use temporary directories to test:

- initialization;
- sequence allocation;
- deterministic slug generation;
- safe creation;
- refusal to overwrite files;
- artifact discovery;
- malformed metadata;
- duplicate identities;
- duplicate sequence numbers;
- deterministic JSON output;
- repository validation;
- failure without content loss.

When practical, write mutations to a temporary file and atomically rename them
into place.

## Output and errors

Human-readable failures should explain:

- what operation failed;
- which artifact or path was involved;
- which invariant was violated;
- what the user can do next.

Automated callers must not need to parse prose.

Do not add placeholder flags. Implement `--json`, `--quiet`, `--dry-run`, and
`--no-interactive` only when they have meaningful behavior.

## Current objective

Prove this vertical slice:

1. locate or initialize a Strata repository;
2. create one safely numbered Markdown dragon;
3. assign it a stable identity;
4. rediscover and parse it;
5. list it in human-readable and JSON forms;
6. report basic repository corruption through `doctor`;
7. dogfood the workflow in this repository.

Do not begin daemon, index, embedding, MCP, or GraphQL work during this sprint.
