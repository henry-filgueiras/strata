---
id: idea-comment-threads
sequence: 11
kind: idea
status: parked
created: 2026-07-21
---

# Durable comment threads anchored to artifacts

## Problem

Multiple agents (human and model) want to hold gradual, durable
discussion about slices of existing documents — a reviewer's critique of
one section of a sprint proposal, a question about one paragraph of a
decision — using the repository as the discussion substrate. Today that
feedback has nowhere to live: editing the target document rewrites
history, logs are not threaded or anchored, and GitHub PR discussion
splits the substrate (requires a remote and an account, is invisible
offline and to grep, anchors to diffs rather than documents, and dies
with the PR context).

Concrete forcing case: an external model reviewed the sprint 2 proposal
and raised a specific objection to the transition safety contract. That
critique wants to attach to one section of `sprint.md` without touching
the file.

## Sketch

A comment thread is an ordinary artifact: one append-only Markdown file
per thread, in a `comments` collection, with lifecycle `open` →
`resolved`. Each entry appended to the thread carries its own header
(entry id, author, created). Append-only files are merge-friendly —
concurrent branches conflict only at the tail, trivially.

The parent document is left byte-identical. This is not a compromise;
decision 6 (`dec-bootstrap-reference-model`) requires it: canonical
files store outgoing references only, and the thread's edge to its
parent is the outgoing reference — stored in the thread's front matter
as a typed edge (`comments-on`, joining the dragon 3 vocabulary with
this as its first consumer). An inline marker in the parent would be a
backlink materialized into a canonical file, which decision 6 already
rules out. Parent-side visibility is a derived projection instead:
`strata show` (or a generated companion view) renders threads as
marginalia. This also eliminates merge conflicts from concurrent
commenters editing the same parent paragraph.

Anchoring uses a graceful-degradation ladder, not character offsets:

- whole document — always valid, the required minimum;
- section heading — stable-ish, cheap;
- quote selector — the W3C Web Annotation model's exact quote plus
  short prefix/suffix context, as used by Hypothesis fuzzy anchoring.

A quote selector is self-describing: if the parent is revised and the
quote no longer matches, the thread becomes *orphaned*, a diagnostic
rather than corruption — and the quoted text inside the anchor still
tells readers what the comment was about. When Git is present, the
anchor may additionally record the blob hash it was made against
(GitHub's "outdated" review-comment model): an always-valid historical
referent, strictly optional so Git remains optional.

Comments advise; they do not define truth. A thread is never
load-bearing: no typed dependency edge may target one, and resolving a
thread invalidates nothing. Conclusions reached in a thread must be
promoted to a decision, dragon, or document revision — the thread is
provenance, not the record. This guards against the familiar
anti-pattern of decisions buried in review conversations.

## Evidence

Decision 6: outgoing-references-only and backlinks-as-projections are
settled, and directly dictate the untouched-parent design. Dragon 3
(`drg_01KY169X7W0YXJ5QFV4D1MK4FB`) owns the marker syntax and typed
edge vocabulary this depends on; `comments-on` fits its
first-consumer-introduces-the-edge rule. Prior art: W3C Web Annotation
Data Model (TextQuoteSelector), Hypothesis anchoring, GitHub review
comments' outdated-anchor semantics. The sprint 2 external review
critique is the live motivating instance.
