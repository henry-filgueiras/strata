---
id: dec-reference-syntax
sequence: 10
kind: decision
status: accepted
created: 2026-07-22
---

# Wikilink reference markers and the first provenance edges

## Context

Decision 6 ([[dec-bootstrap-reference-model|reference model]]) settled
reference semantics and promoted two open points — concrete marker
syntax and the initial typed edge vocabulary — to dragon 3
([[drg_01KY169X7W0YXJ5QFV4D1MK4FB|reference syntax and vocabulary]]),
which this decision resolves. The constraints it inherits: one grammar
with two strictness levels, stable-ID targets with frozen labels,
typed edges in front matter versus untyped markers inline, and the
acceptance test that any candidate must read acceptably in a raw
GitHub pull-request diff with no tooling installed (decision 7,
[[dec-bootstrap-interaction-surfaces|interaction surfaces]]).

## Decision: marker grammar

Inline untyped references use wikilink-style markers:

- **Bound (canonical):** `[[stable-id|label]]`. The target is a stable
  artifact ID; the label is mandatory, frozen at bind time, and
  non-authoritative decoration. Convention: the target's title or a
  compact paraphrase of it.
- **Unbound (sugar):** `[[kind:N]]`, optionally `[[kind:N|label]]`. The
  target is a collection-scoped sequence reference, exactly the form
  intent commands accept (`dragon:3`, `idea:12`). Legal but weak;
  repaired by an explicit bind that rewrites the target to the stable
  ID and freezes a label, in place, inside the same delimiters.

One grammar, two strictness levels: a target containing a colon is
sugar, anything else is a stable ID. Binding is a repair within the
language: `[[dragon:3]]` becomes
`[[drg_01KY169X7W0YXJ5QFV4D1MK4FB|reference syntax dragon]]`.

Grammar details:

- A marker is `[[` target (`|` label)? `]]` on one line. The label runs
  from the first `|` to the closing `]]` and may contain anything but
  `]]` and newlines.
- Reference targets must not contain `|`, `#`, `]]`, or whitespace.
  `#` is reserved unused: dragon 3's sub-artifact fragment question
  (comment-thread entries) is **deferred**, and reserving `#` in the
  target grammar makes the future fragment extension non-breaking.
  Consequence of deferral: thread entries remain citable only as prose
  URL fragments (`0001-thread.md#entry-id`) until a decision adopting
  idea 11 extends the grammar and defines orphaning semantics.
- Text inside fenced code blocks is never a marker, mirroring how
  title extraction already treats fences.

Mnemonic for the two syntactic homes: **prose points, front matter
promises.**

## Decision: why not Markdown-link style

The rejected candidate was `[label](strata:stable-id)`. It fails the
readability test at the rendered layer rather than the raw one:
GitHub's HTML sanitizer allows only a small allowlist of URI schemes,
so a `strata:` anchor is stripped and the reference renders as bare
label text — the reference disappears exactly where humans read most.
A wikilink renders literally everywhere: noisier, but it never hides
that a reference exists or what it targets, which is the property a
repository-must-not-be-hostage tool actually needs. Clickability is a
job for future read-side projections, not for the canonical bytes.

Secondary counts against it: overloading real Markdown link syntax
makes reference-shaped text ambiguous with ordinary URLs (decision 6
requires markers to be explicit, inferring nothing from prose), and
the sugar form nests colons awkwardly (`[dragon 3](strata:dragon:3)`)
where the wikilink sugar is minimal (`[[dragon:3]]`). Neither
candidate escapes YAML quoting in front matter — both begin with `[`,
which YAML reads as a flow sequence — so that was no tiebreak.

## Decision: typed edge encoding and initial vocabulary

A typed edge is a front-matter field whose value is a **bound** marker
as a quoted YAML string, or a YAML sequence of them when one kind has
several targets:

```yaml
resolved-by: "[[dec-reference-syntax|reference syntax decision]]"
```

Same grammar as prose, one more strictness level: sugar in a typed
edge value is legal but weak (bindable, not verifiable), and doctor
reports it as repairable rather than corrupt.

The initial vocabulary is deliberately terminal-provenance only — each
kind lands with its first consumer in sprint 4, per dragon 3's
introduce-with-consumer constraint:

- **`resolved-by`** — source: a dragon, expected on closed ones;
  target: a decision or task that settled it; one or more targets.
- **`adopted-by`** — source: an idea, expected on adopted ones;
  target: the decision or task that adopted it.

The shared rule both instantiate: *terminal lifecycle states carry a
provenance edge to the work that terminated them.* The obvious next
kind, `rejected-by`, waits for its first rejected specimen; dependency
kinds (`depends-on`, `blocked-by`) wait for managed sprints and idea
14's validity rules. Adopting decisions may still cite ideas as
motivating provenance — inline, untyped, per the never-load-bearing
rule.

## Decision: doctor semantics

The verification universe is every front-matter `id` in the
archaeology tree, managed collections or not, so edges may target
decisions and tasks before those become managed. Per edge kind:

- **Corruption (error):** a value that does not parse as a marker; a
  bound target absent from the universe (dangling); a target whose
  `kind` the edge forbids — for both v1 kinds anything but `decision`
  or `task`, which also structurally enforces that no typed edge
  targets an idea.
- **Repairable (diagnostic):** an unbound sugar value; an edge on a
  source whose lifecycle state contradicts it, such as `resolved-by`
  on an open dragon — a stale claim to investigate, not corruption.
- **Advisory (out of deterministic scope):** *absence* — a closed
  dragon or adopted idea missing its provenance edge. Enforcing
  presence would turn every bare `strata close` into an instant red
  doctor; promotion to a strict tier is idea 13's question
  ([[idea-strict-doctor|strict doctor]]).
- Front-matter keys outside the decided vocabulary are inert data,
  ignored by doctor; the vocabulary is a closed allowlist grown only
  by decision. Dangling *untyped* markers remain diagnostic at most,
  and checking them at all stays deferred with idea 2
  ([[idea-doctor-reference-graph|reference-graph checks]]).

## Consequences

- The interim both-forms prose convention is retired for writing that
  postdates this decision; existing prose references stay untouched
  except where a task already rewrites the file.
- Writers without tooling stay unblocked: sugar is legal everywhere,
  and the repair (bind) is a textual widening a human can perform by
  hand.
- Front-matter edge values require YAML quotes; forgetting them parses
  as a nested flow sequence and doctor reports the malformed value as
  corruption, naming the file.
- `strata close` gains an obvious future flag (`--resolved-by`) so the
  provenance edge can ride the transition; recorded as an observation,
  not committed work.
