---
id: ide_01KY7R7CA8FNBRH3DFKFZW8V6J
sequence: 22
kind: idea
status: parked
created: 2026-07-23
---

# Reviewable mutation intents across authority boundaries

## Problem

A contributor can read the archaeology, understand its invariants, and
formulate a valid intent-level operation — park this idea, open that
dragon — while lacking, or deliberately not being granted, authority to
mutate canonical files. Today the handoff is either unstructured prose
that a mutation-capable session must re-derive into an operation, or
write access broader than the contributor actually needs.

The forcing case is narrow and honest: one external, effectively
read-only Grok review produced an RFC that Henry had to carry by hand
into the mutation-capable workflow. One specimen justifies parking the
interface boundary, not building machinery.

Write-capable contributors are not the audience. They already have the
worktree, branches, diffs, and pull requests as review surfaces; this
idea must not duplicate Git.

## Sketch

A versioned, machine-readable mutation-intent envelope representing an
operation Strata already understands. It may carry:

- the requested operation and its semantic arguments;
- authored body content where applicable;
- contributor and provider provenance;
- optional expectations about observed repository state.

A trusted executor reviews the envelope and applies it through the same
semantic core the normal CLI uses. Final artifact identity, display
sequence, and path are allocated only at application time, against
current repository state — never promised earlier. A request may carry
its own request identity for transport or replay protection, but that
identity is distinct from the resulting artifact identity. Exact
schema, persistence, idempotency, and command names remain open until
real recurrence supplies evidence.

This extends
[[idea-capability-constrained-work|Capability-constrained work]]:
permissions attach to the invoking session's affordances, not
to "human" versus "agent" — and formulating an effect and authorizing
that effect are separate capabilities.

## Boundaries

- No `proposals/` collection is proposed yet.
- No `proposed` status is added to dragons, ideas, tasks, or any other
  lifecycle.
- `strata propose idea` is avoided: a parked idea is already a
  proposal.
- Pending requests are not doctor findings; doctor remains structural
  validation.
- A CLI subcommand alone is not an ACL or a security boundary.
- A preview must not promise that a final sequence, path, or artifact
  ID has been reserved.
- Application must reuse the existing operation core, not grow
  parallel human and agent semantics.
- The envelope initially expresses only operations Strata actually
  supports; it must not smuggle in `new decision`, `new log`, or other
  nonexistent operations.
- MCP, generalized agent protocols, autonomous action, and multi-agent
  locking remain deferred.
- No implementation work is justified until repeated read-only or
  constrained-agent handoffs demonstrate real friction.

## Evidence

Originating RFC: Grok (xAI), 2026-07-22, proposing `strata propose`, a
repository-local `proposals/` staging area, `status: proposed`, doctor
surfacing, and later `adopt`/`apply` commands. Review retained its
authority-separation kernel and rejected the staging collection and
lifecycle expansion. For callers that already hold write authority,
Git branches and working-tree diffs are sufficient prior art; the
distinct use case is a mediated boundary where one session may
formulate requests and another may authorize effects.
