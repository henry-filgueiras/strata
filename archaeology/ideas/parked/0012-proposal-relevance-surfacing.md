---
id: idea-proposal-relevance-surfacing
sequence: 12
kind: idea
status: parked
created: 2026-07-21
---

# Proposal-time relevance surfacing

## Problem

The archaeology's value is bounded by its read rate, and unaided recall
does not scale in either direction. A human will not reread a growing
corpus before every proposal; an agent's in-context matching only works
over whatever subset of the corpus is loaded, and the corpus will
outgrow any context window. The failure mode is silent: a new proposal
quietly re-litigates a settled tradeoff or walks into a recorded dragon,
and nobody notices because nothing *surfaced* the relevant history.

Henry's framing (2026-07-21): being told "this file explains why this
won't work, take a look" earns an honest read that a standing
obligation to search never will. Finding related history at the moment
a proposal arrives is one of the holy grails.

## Sketch

Given a new proposal P — an idea, a sprint draft, a decision in
progress — retrieve the likely-related dragons, decisions, tasks, and
evidence; assess whether that set supports, conflicts with, or is
silent on P; and bubble the result up to whichever agents (human or
model) are designated to make the steering call.

Constraints already settled elsewhere:

- Advisory only. Semantic systems advise; they do not define truth.
  Surfaced relevance is a prompt to read, never a structural validation
  failure, and it mutates nothing without an explicit operation.
- Any index over the corpus is a disposable projection, rebuildable
  from canonical files.
- Retrieval's real job is context selection: it decides what enters an
  agent's working context, complementing rather than duplicating the
  agent's own in-context reasoning.

Mechanism is deliberately unspecified — lexical overlap over
front-matter and references might embarrass embeddings for a corpus
this shaped, and the reference graph (decision 6) is itself a retrieval
signal. Fortune (idea 6) is the degenerate ancestor: push one stale
artifact with no query. This idea is fortune with a query.

## Evidence

Embeddings and semantic search are explicit bootstrap non-goals;
parking this is how that boundary is honored without losing the
articulation. CLAUDE.md names semantic retrieval as a possible eventual
projection. The read-rate concern is recorded in sprint 2's rationale
("is this a memory or a diary?"). Prior art: code-review bots that
surface related PRs/issues, IDE "similar code" hints, and the general
retrieval-augmented-generation pattern, all advisory-by-construction.
