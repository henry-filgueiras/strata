---
id: ide_01KY7S6GMN26BFTEVGGKZHN4ZC
sequence: 26
kind: idea
status: parked
created: 2026-07-23
---

# Managed amendments: dated in-place supersession

## Problem

The preserve-history invariant means canonical artifacts change by
appending dated sections, never by rewriting: the decisions corpus
already carries seven such sections, sprint 6 amended its own rationale
the same way, and the pattern is load-bearing — CLAUDE.md cites
"decision 11 as amended" as authority. Yet the mechanism is entirely
free-form. Two heading grammars have already diverged
(`## Update (date): title` and `## Amendment: title (date)`), nothing
marks which earlier text an amendment supersedes, and a reader must
diff headings by eye to discover that a decision has drifted from its
original statement. The most authority-bearing operation in the
repository is the least structured one.

## Sketch

Make the amendment a first-class operation: `strata amend <ref>
"<title>"` appends a scaffolded dated amendment section, and possibly
records an `amended:` date list in front matter so `list` and `show`
can flag amended artifacts without reading the body. Convention before
command is acceptable and may be the whole first slice: a recorded
grammar for the heading, adopted by decision, captures most of the
value; the command mechanizes it later if recurrence justifies.

## Boundaries

- No history rewrite, no versioning system, no diff storage — Git
  already owns textual history.
- Amendment does not change lifecycle status: an amended decision is
  still `accepted`; superseding a decision outright is a different,
  future operation.
- Old free-form sections are not migrated; the grammar binds new
  writing only, matching how [[dec-reference-syntax|decision 10]]
  handled prose references.

## Evidence

Seven dated update/amendment sections across four decision files, in
two divergent grammars; sprint 6's rationale amendment; decision 14's
post-close narrowing (task 31), which is the pattern operating at its
highest stakes — correcting an incident's own output. Each was
hand-typed with no scaffold and no structural trace.

Proposed by Claude during the sprint 7 pitch, 2026-07-23.
