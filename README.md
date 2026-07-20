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
