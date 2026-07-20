# Historical one-shot inception script.
#
# This recreates Strata's original manually seeded repository layout.
# It is not the implementation of `strata init` and is not intended for
# routine use against an existing repository.
#
# Retained to document and reproduce the repository's original seed state.
# Do not update this script to mirror later repository evolution.

#!/usr/bin/env bash
set -euo pipefail

if [[ ! -d .git ]]; then
  echo "error: run this from the root of the cloned Strata repository" >&2
  exit 1
fi

MARKER="meta/bootstrap_complete"

if [[ -e "$MARKER" ]]; then
  echo "error: Strata inception bootstrap has already completed" >&2
  exit 1
fi

if [[ -n "$(git status --porcelain)" ]]; then
  echo "error: working tree is not clean; refusing to seed over existing work" >&2
  exit 1
fi

echo "==> Normalizing local branch to main"
current_branch="$(git branch --show-current)"
if [[ "$current_branch" != "main" ]]; then
  git branch -M main
fi

echo "==> Initializing Rust binary crate"
cargo init --bin --name strata .

mkdir -p \
  archaeology/decisions \
  archaeology/dragons/open \
  archaeology/dragons/closed \
  archaeology/logs \
  archaeology/sprints/0001-bootstrap/pending \
  archaeology/sprints/0001-bootstrap/closed

cat > README.md <<'EOF'
# Strata

**Git-native project archaeology and structured repository memory for humans and coding agents.**

Strata is an experimental command-line tool for maintaining durable project
knowledge as ordinary repository files.

Its intended artifacts include:

- architectural decisions
- unresolved technical risks, or “dragons”
- investigations and development logs
- sprints and work items
- structured JSON documents
- row-oriented JSONL datasets

The filesystem remains canonical. Git provides history. Strata supplies safe,
intent-level operations, validation, and machine-readable projections.

## Bootstrap status

Strata is currently bootstrapping its smallest useful vertical slice:

1. initialize a Strata repository;
2. create a numbered Markdown artifact safely;
3. rediscover artifacts deterministically;
4. list them in human-readable and JSON forms;
5. report malformed or conflicting repository state.

Daemon, indexing, semantic search, embeddings, MCP, GraphQL, and dashboards are
deliberately deferred.

## Development

```sh
cargo build
cargo test
cargo run -- --help
```

See [`CLAUDE.md`](CLAUDE.md) for project invariants and agent workflow.
EOF

cat > CLAUDE.md <<'EOF'
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
EOF

cat > archaeology/decisions/0001-files-are-canonical.md <<'EOF'
---
id: dec-bootstrap-files-canonical
sequence: 1
kind: decision
status: accepted
created: 2026-07-20
---

# Files are canonical

## Context

Strata needs durable project memory that humans, Git, shell tools, and coding
agents can all inspect.

A database could simplify querying, but it would make repository understanding
depend on Strata-specific storage and tooling.

## Decision

Canonical project records are ordinary repository files.

Databases, indexes, generated summaries, dashboards, and embeddings may exist
only as disposable projections that can be rebuilt from canonical files.

## Consequences

- Git history, review, branching, blame, and revert work naturally.
- Repositories remain legible without Strata.
- File layout and mutation safety become important correctness surfaces.
- Query acceleration may later require a rebuildable index.
EOF

cat > archaeology/decisions/0002-stable-identity-and-display-sequence.md <<'EOF'
---
id: dec-bootstrap-stable-identity
sequence: 2
kind: decision
status: accepted
created: 2026-07-20
---

# Separate stable identity from display sequence

## Context

Sequential filename prefixes make artifacts easy to sort and reference:

```text
0007-wasm-stack-pressure.md
```

Two concurrent Git branches may independently allocate the same next number.
Renaming artifacts during merge should not change their durable identity.

## Decision

Artifacts will eventually carry both:

- a stable machine identity, likely a ULID;
- a collection-scoped display sequence used in filenames and human references.

The exact metadata schema remains provisional during bootstrap.

## Consequences

- Filename sequences may be repaired after branch collisions.
- Links and machine operations should prefer stable identities.
- Humans retain compact sortable filenames.
EOF

cat > archaeology/decisions/0003-payload-format-is-not-the-artifact-model.md <<'EOF'
---
id: dec-bootstrap-payload-separation
sequence: 3
kind: decision
status: accepted
created: 2026-07-20
---

# Payload format is not the artifact model

## Context

Narrative decisions and investigations fit Markdown well.

Other repository evidence is naturally represented as structured JSON or
row-oriented JSONL, including benchmark runs, performance samples, traces, and
experiment observations.

Forcing every artifact into semi-structured Markdown would weaken machine
readability and encourage duplicated sources of truth.

## Decision

Artifact identity, collection, lifecycle, and placement are independent of
payload representation.

Markdown is the first supported codec, not the universal storage format.

JSON and JSONL support are intentionally deferred until the Markdown workflow
is proven.

## Consequences

- Core abstractions must not embed Markdown-specific assumptions unnecessarily.
- Machine-friendly evidence can remain canonical.
- Human-readable tables and charts may later be generated projections.
EOF

cat > archaeology/dragons/open/0001-branch-sequence-collisions.md <<'EOF'
---
id: drg-bootstrap-branch-collisions
sequence: 1
kind: dragon
status: open
created: 2026-07-20
---

# Branch sequence collisions

## Context

Two branches can independently inspect the same collection, choose the same next
display sequence, and create different artifacts with identical numeric
prefixes.

## Question

What repair policy should Strata use when duplicate display sequences are found
after branches merge?

## Constraints

- Stable artifact identities must not change.
- Existing artifacts should not be renumbered casually.
- Repair must never overwrite or lose content.
- `doctor` should detect collisions deterministically.

## Candidate direction

Treat sequence numbers as repairable presentation metadata. Provide an explicit
future repair operation rather than silently renumbering during ordinary reads.

## Resolution criteria

Resolve after the bootstrap scanner and validator make the collision behavior
concrete enough to test.
EOF

cat > archaeology/sprints/0001-bootstrap/sprint.md <<'EOF'
---
id: spr-bootstrap
sequence: 1
kind: sprint
status: active
created: 2026-07-20
---

# Sprint 1: Bootstrap

## Goal

Create one artifact safely, rediscover it deterministically, and report basic
repository corruption clearly.

## Success criteria

A user can run:

```sh
strata init
strata new dragon "Example unresolved risk"
strata list dragons
strata list dragons --json
strata show dragon:2
strata doctor
```

The implementation must:

- refuse destructive overwrites;
- assign a stable identity;
- choose the next display sequence;
- generate a deterministic safe slug;
- parse artifacts it created;
- expose deterministic JSON;
- diagnose malformed or conflicting state;
- preserve content when operations fail.

## Non-goals

This sprint does not implement:

- configurable arbitrary collections;
- JSON or JSONL CRUD;
- lifecycle transitions;
- Git integration;
- indexing or watchers;
- embeddings;
- MCP or GraphQL;
- dashboards or TUIs.
EOF

cat > archaeology/sprints/0001-bootstrap/pending/0001-define-cli-and-errors.md <<'EOF'
---
id: tsk-bootstrap-cli-errors
sequence: 1
kind: task
status: pending
sprint: spr-bootstrap
created: 2026-07-20
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
EOF

cat > archaeology/sprints/0001-bootstrap/pending/0002-initialize-repository.md <<'EOF'
---
id: tsk-bootstrap-init
sequence: 2
kind: task
status: pending
sprint: spr-bootstrap
created: 2026-07-20
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
EOF

cat > archaeology/sprints/0001-bootstrap/pending/0003-create-dragon.md <<'EOF'
---
id: tsk-bootstrap-create-dragon
sequence: 3
kind: task
status: pending
sprint: spr-bootstrap
created: 2026-07-20
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
EOF

cat > archaeology/sprints/0001-bootstrap/pending/0004-list-and-show-artifacts.md <<'EOF'
---
id: tsk-bootstrap-list-show
sequence: 4
kind: task
status: pending
sprint: spr-bootstrap
created: 2026-07-20
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
EOF

cat > archaeology/sprints/0001-bootstrap/pending/0005-validate-repository.md <<'EOF'
---
id: tsk-bootstrap-doctor
sequence: 5
kind: task
status: pending
sprint: spr-bootstrap
created: 2026-07-20
---

# Validate repository invariants

## Objective

Implement `strata doctor` for the bootstrap artifact model.

## Acceptance criteria

Detect and report:

- malformed front matter;
- metadata inconsistent with file placement;
- duplicate stable identities;
- duplicate display sequences;
- invalid filenames;
- unreadable files.

Validation must not modify canonical files during this sprint.
EOF

cat > archaeology/logs/0001-project-inception.md <<'EOF'
---
id: log-bootstrap-inception
sequence: 1
kind: log
created: 2026-07-20
---

# Project inception

Strata began as an attempt to replace growing omnibus development-note files
with individually addressable repository artifacts.

The initial hypothesis is that a small filesystem protocol can improve both
human project continuity and coding-agent handoffs without requiring a
centralized project-management database.

Potential later layers include rebuildable indexing, semantic retrieval, file
watching, MCP integration, and generated projections. These remain explicitly
outside the bootstrap scope.
EOF

cat > rustfmt.toml <<'EOF'
edition = "2024"
EOF

mkdir -p meta
cat > "$MARKER" <<'EOF'
Strata repository inception bootstrap completed.

This marker prevents accidental reruns of scripts/bootstrap-inception.sh.
The script records the original seed state and is not kept synchronized with
the repository's later evolution.
EOF

echo
echo "Seed complete."
echo
echo "Suggested next commands:"
echo "  cargo fmt"
echo "  cargo test"
echo "  git status"
echo "  git add ."
echo "  git commit -m 'chore: bootstrap Strata repository'"
echo "  git push -u origin main"
