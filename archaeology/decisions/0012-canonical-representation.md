---
id: dec-canonical-representation
sequence: 12
kind: decision
status: accepted
created: 2026-07-22
---

# Canonical representation and identity addressability

## Context

The sprint 5 post-merge review adjudicated two coupled defects. Thread
5 ([[cmt-s5-global-identity-catalog|global identity catalog]]) showed
that identity uniqueness is checked over the strongly managed universe
while typed-edge resolution trusts a best-effort harvest of the whole
tree, so id collisions straddling that gap resolve silently and
first-wins. Thread 6 ([[cmt-s5-operability-closure|operability
closure]]), cases B, C, and E, showed that the read model admits
representations other surfaces refuse or corrupt: quoted and
comment-bearing status spellings pass doctor but are untransitionable,
ids the read model accepts cannot be addressed by the CLI or the
marker grammar, and provenance binding freezes titles into markers it
never re-parses.

Tasks [[tsk_01KY62E9VMB6HDNJWD31YS1FBP|task 23]] (identity catalog),
[[tsk_01KY6364DV39W0DZ3N0NF8GBGB|task 25]] (representation contract),
and [[tsk_01KY640RFXZJMWZ2T8W9B628AA|task 27]] (corpus operability)
each require a recorded contract before implementation. This decision
is that contract's representation half: what constitutes an identity
claim, which claims are addressable, what a marker label may contain,
and which mutable representations the write path recognizes. It
layers on decision 2 ([[dec-bootstrap-stable-identity|stable
identity]]), decision 4 ([[dec-bootstrap-error-contract|error
contract]]), and decision 10 ([[dec-reference-syntax|reference
syntax]]); it amends none of them.

## Decision: claim admission is not addressability

An identity claim is admitted by exactly the universe thread 5
adjudicated: a file whose bounded content is valid UTF-8, whose
front-matter framing parses, whose front matter is YAML-parseable,
and whose front matter carries a string `id` and a string `kind`.
Nothing else is required — not status, sequence, date, placement, or
a known kind. An `id` without a string `kind` is not a claim. This
threshold is settled review authority, not an open engineering
choice; widening or narrowing it requires new evidence and a new
decision.

Every admitted claim is retained in the claimant catalog, including
when:

- the rest of the artifact is malformed under the canonical parse;
- the claimed id is not addressable through the CLI or the marker
  grammar;
- the file is unmanaged;
- another claimant carries the same id.

Addressability is applied during contract validation or resolution.
It is never a harvest filter: task 25's contract decides which
identities may be bound, while task 23's catalog sees every
syntactic claimant. Filtering the harvest by addressability would
re-create the exact invisibility thread 5 convicted — a claimant you
cannot see is a claimant something will silently resolve against.

## Decision: claimant disposition is explicit

The catalog, or a single typed joined view over it, preserves an
explicit disposition per claimant sufficient to distinguish:

- **canonical** — the claimant passes the canonical artifact parse;
- **probe-only or otherwise unassessed** — the claim was admitted but
  no canonical-parse verdict is recorded for it;
- **rejected by canonical parsing** — the canonical parse refused the
  file, with a stable reason class when one is known.

Optional harvested fields such as `sequence` or `title` are not an
adequate substitute for this disposition. Callers must not infer
validity from missing optional fields, and must not reclassify parser
failures in ad hoc ways; the disposition travels with the claimant or
through the one joined view, nowhere else.

This requirement fixes information content, not implementation: it
does not freeze Rust type names, and it does not require retaining
payloads — catalog entries remain bounded metadata per task 23's
scope clarification. Aggregate loading and caching remain owned by
thread 8 ([[cmt-s5-read-cost-and-watermark|read cost and watermark]])
and idea 18 ([[ide_01KY5X7C56KBFWJJJKHTEXXQXV|modification
watermark]]).

Claimants of one id are ordered by repository-relative path. The
regression pinning determinism must build the same path set under
opposite insertion and enumeration orders and assert both runs yield
the same path-sorted claimant list.

## Decision: addressable stable ids

Decision 2's floor is untouched: every non-empty historic id remains
a valid identity forever, and no reader may require ULID structure.
This decision defines only the subset of identities usable as
stable-id addresses (CLI id arguments) and as marker targets — an
explicit narrowing of the read model's documented "any non-empty
string" promise, which task 25 requires be recorded rather than
implied.

An id is addressable when it is non-empty and contains none of the
following character classes, judged over the decoded string value:

- **`:`** — a colon-bearing target is sugar by decision 10's grammar,
  and the CLI routes every `:`-bearing reference to the `kind:N`
  parser; such an id can never be reached by the id arm.
- **whitespace** (any character with the Unicode whitespace property,
  which includes newlines) — recorded marker-target grammar.
- **`#`** — reserved in the target grammar for future fragments.
- **`|`** — the marker's target/label separator.
- **`]`** — a deliberate narrowing owned by this decision. Decision
  10 excludes only `]]` from targets; excluding the single `]` as
  well keeps the target/label boundary unambiguous under the
  suffix-anchored label parse below and costs nothing observed in
  practice. This is not a claim that decision 10 already excluded it.

Addressability operates on the decoded value: `id: x` and `id: "x"`
both claim the semantic id `x`, and two files spelling it those two
ways collide. YAML quoting of an id is not noncanonical — no accepted
criterion requires a raw carrier spelling for ids — so doctor must
not flag it and the catalog must not distinguish it.

Non-addressable ids remain valid identities, remain catalogued,
and remain diagnosable: doctor reports them on otherwise parseable
artifacts so that doctor-green implies addressable, and binding
refuses them before mutation. Every generated id shape and every
hand-authored id currently in this corpus (`dec-*`, `drg-*`,
`idea-*`, `log-*`, `spr-*`, `tsk-*`, `cmt-*`, and the prefixed ULID
forms such as `drg_*`) is addressable under these rules.

## Decision: marker labels — decision 10 implemented as written

The recorded default is taken: decision 10's label grammar stands and
`parse_marker` is brought into agreement with it. Decision 10 is not
amended.

- A label may contain a single `]`; it may not contain `]]` or a
  newline.
- Label parsing is suffix-anchored: the label runs from the first `|`
  to the final `]]` that closes the marker, so a label ending in one
  `]` round-trips. Today's typed-edge parse (`strip_suffix("]]")`)
  already demonstrates the shape; `parse_marker`'s reject-any-`]`
  shortcut is the side that moves.
- `resolve_edge` validates the marker it constructs — round-tripping
  it through the parser — before any mutation. A target whose id or
  frozen title cannot form a parseable marker is refused with a typed
  error naming the offending character class, and the source
  artifact's bytes are unchanged.

Caveat, recorded rather than solved: a label ending in `]` makes the
raw text end in `]]]`. A future prose scanner that searches for the
first `]]` would misparse such a marker; when prose scanning is
built, it must anchor on the closing delimiter the way the typed-edge
parse does, or the label grammar must be revisited then, with that
scanner as evidence. Silently tightening decision 10 now to
pre-solve a hypothetical scanner is exactly the kind of undocumented
divergence thread 6 case E convicted.

## Decision: canonical status carrier

The transition splicer and doctor share one recognizer for the
mutable `status` carrier, so the two surfaces cannot drift. The
recognized form, within the front-matter block only:

- exactly one line beginning `status:` at column zero — duplicate
  `status:` lines are refused, and an indented spelling is not a
  status carrier at all;
- the remainder of the line, with surrounding whitespace trimmed, is
  exactly one admitted lowercase status word for the collection;
- quoting (single or double) is not recognized;
- inline comments are not recognized;
- whitespace variation around the value is accepted and preserved
  byte-for-byte by the splice — write sites nevertheless always emit
  the one spelling `status: <word>`.

Canonical parsing and representation conformance remain distinct
layers:

- the canonical artifact parse provides command-readable semantics
  (`status: "open"` still reads as open);
- doctor reports parseable-but-nonconforming mutable representations
  at error severity, so doctor-green implies transitionable;
- write and bind sites enforce the contract they own: creation and
  transition never write an excluded representation, and the splicer
  keeps refusing rather than guessing — its refusal text names the
  canonical spelling instead of referring to a doctor that reports
  nothing;
- line-ending posture belongs to
  [[tsk_01KY6364E105F7AWT7RAZ264WZ|task 26]]; title renderability and
  creation rollback belong to
  [[tsk_01KY6364DMJ7DPEWCAK0ZKDNHR|task 24]]. This decision absorbs
  neither.

## Decision: diagnostic categories

New findings land in doctor's provisional `problem` vocabulary as
decision 4 already provides: the managed-only `duplicate-id` check is
subsumed by one catalog-wide finding naming every claimant path; an
edge bound to an ambiguous id draws an error finding (working name
`ambiguous-edge`); a contract-excluded representation on an otherwise
parseable artifact draws an error finding distinct from
`malformed-artifact` (working name `non-canonical-artifact`, since
the file parses but is excluded). Working names are provisional per
decision 4; the frozen surfaces — exit codes and `error[<code>]`
categories — are untouched, and binding refusals reuse the existing
`ambiguous-reference` (exit 8) and `artifact-not-found` (exit 7)
categories.

## Out of scope: the unique malformed provenance target

Whether a unique id whose only claimant is malformed or unmanaged may
serve as a provenance target is the compatibility seam task 23
expressly records as out of scope. Current behavior — such a bind may
succeed when a title is extractable — is preserved unchanged. This
record does not bless it as desirable policy and does not repair it;
evidence either way belongs in task 23's result.

## Alternatives rejected

- **Admitting `id` without `kind` as a claim.** Reopens thread 5's
  adjudicated universe without evidence, and a kindless claimant
  cannot participate in kind-checked edge validation without
  inventing a kind for it.
- **Deriving claimant validity from optional fields.** Absent
  `sequence` or `title` is a fact about what the probe recovered, not
  a verdict; treating it as one is precisely the ad hoc
  reclassification this contract forbids.
- **Amending decision 10 to forbid `]` in labels.** Tidier for a
  future prose scanner, but it amends an accepted decision to
  pre-solve a hypothetical consumer; the recorded default is
  implement-as-written, with the caveat above preserving the evidence
  trail for revisiting.
- **Declaring quoted ids noncanonical.** No authority requires a raw
  carrier spelling for ids; collision semantics over decoded values
  already give the right answer, and flagging quoting would invent a
  conformance rule with no operational consequence.
- **Enforcing status conformance in the strict read.** Making
  `status: "open"` a read failure would strand every command on a
  sibling that is today merely nonconforming — widening exactly the
  degradation thread 6 case G convicted. Enforcement lands at doctor
  and the write and bind sites instead.

## Consequences

- The read model's id documentation is narrowed explicitly; identity
  validity is unchanged, addressability is the new named subset.
- `parse_marker` moves to the suffix-anchored label parse;
  `resolve_edge` validates constructed markers before mutating; the
  splicer's refusal text is repaired; doctor gains the conformance
  and catalog findings — all under tasks 23 and 25, which implement
  this contract and cite it.
- Task 27's operability policy composes over this contract: what a
  "malformed sibling" or an "admitted representation" means below is
  what this record defines.
- The existing corpus conforms as-is: no migration, no rewritten
  ids, no moved files.
