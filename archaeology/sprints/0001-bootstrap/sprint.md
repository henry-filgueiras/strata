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
