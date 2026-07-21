---
id: spr-bootstrap
sequence: 1
kind: sprint
status: closed
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

## Retrospective (2026-07-21)

All six tasks closed; every success criterion above is exercised by the
test suite and dogfooded in this repository, which now maintains its own
dragons with the tool. The sprint produced seven decisions, raised three
dragons and closed one, and parked eight ideas.

Durable learnings, recorded where they belong:

- repository validity must be closed under Git round-trip; the marker
  alone defines the repository (decision 0005 update, dragon 0002);
- scope calls made without enough evidence can land as explicitly
  provisional decisions instead of blocking or silently freezing —
  the doctor finding vocabulary is the worked example (decision 0004
  update, task 0005 result);
- the archaeology predicted its own critical path: dragon 0002 named
  task 0005 as the work it gated before either was implemented, and
  sequencing fell out mechanically — motivating evidence recorded in
  idea 8 (`idea-frontier-projection`).

Friction to fix next: lifecycle transitions were performed by hand three
times this sprint (dragon 0002, task 0005, this closure), each a
move-plus-status edit that doctor can check but no command yet owns.
