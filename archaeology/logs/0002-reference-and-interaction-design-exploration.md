---
id: log-bootstrap-reference-exploration
sequence: 2
kind: log
created: 2026-07-20
---

# Reference model and interaction surface exploration

A design conversation following task 0004 explored how artifacts should
reference one another and what surface users interact through. The settled
principles were recorded as decisions 0006 (reference model) and 0007
(interaction surfaces). This log preserves the alternatives that were
considered and rejected, so the tradeoffs are not reopened without new
evidence, and parks the future ideas the discussion produced.

## Rejected alternatives

- **Union of two reference syntaxes for the same edge kind** (front matter
  and prose both carrying untyped links, merged by tooling). Rejected:
  drift between the two locations becomes invisible exactly because the
  union operator hides it. Instead, edge *kinds* are partitioned — typed in
  front matter, untyped inline (decision 0006).
- **Inferring references from unmarked prose.** Rejected: negation,
  quotation, and hypotheticals make prose a hostile substrate; "does not
  supersede decision 3" contains the string an extractor would match.
- **References targeting display sequences.** Rejected: inherits the
  branch-collision problem (dragon 0001) into the reference graph.
- **Hand-maintained bidirectional links.** Rejected: one side always rots;
  backlinks are a derived projection instead.
- **CI or hooks performing reference binding.** Rejected: late binding
  resolves sugar against a different tree than the author's intent and can
  silently bind to the wrong artifact; automation verifies binding
  (blocking, fmt-check style) but never performs it.
- **Continuous label reconciliation** (automation rewriting stale human
  labels in canonical files). Rejected as cosmetic churn on historical
  records; labels are frozen decoration, projections display current
  truth, and batch rewrites happen only inside explicit repair operations.
- **`.gitkeep` files to preserve empty managed directories.** Rejected as
  symptom-level for the git round-trip flaw (dragon 0002); the candidate
  fix is marker-only validity with lazy directory materialization.
- **"Strata as browser" framing** (raw Markdown demoted to view-source).
  Rejected for narrative artifacts; recorded as decision 0007. The framing
  is considered correct for structured JSON/JSONL payloads.

## Parked future ideas

None of these are scheduled; each needs a concrete consumer and its own
decision before implementation.

- `strata links bind` (resolve unbound sugar references to stable IDs) and
  `strata links bind --check` as a local/CI gate.
- Doctor checks over the derived reference graph: dangling typed edges as
  likely corruption, unbound sugar and dangling untyped markers as
  diagnostics, label drift as information.
- `strata edit`: `$EDITOR` round-trip with validation and binding on
  return, managed front-matter fields protected as scaffolding; the
  editing story for structured payloads.
- Editor integrations (completion for references) as thin shims over the
  CLI core.
- A typed `resolved-by`/`supersedes` edge between dragons and the
  decisions that settle them, replacing today's prose promises.

## Process note

The archaeology layout has no designated home for future ideas, although
the change-discipline rules require recording them; this log is the
stand-in. If idea-parking recurs, the gap deserves a real answer (a
collection, or a convention) rather than more stand-ins.

## Update (2026-07-20): parking-lot role retired

Task 0006 (`tsk-bootstrap-ideas-collection`) established
`archaeology/ideas/`; the parked ideas above were migrated to individual
artifacts (ideas 1 through 5). This log remains the canonical record of
the rejected alternatives; the "Parked future ideas" section above is
historical and no longer maintained.
