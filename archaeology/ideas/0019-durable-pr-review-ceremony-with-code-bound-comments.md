---
id: ide_01KY5YG15T64AA6K5F0VVDJT97
sequence: 19
kind: idea
status: parked
created: 2026-07-22
---

# Durable PR review ceremony with code-bound comments

## Problem

"Pitch amended in review" has no durable trace anywhere in the
industry's default toolchain: the conversation that reshapes a change
before it merges lives in PR comments — unversioned, platform-bound,
never auditable next to the code it shaped, and invisible to the
archaeology. Twice now a Strata sprint's first task was materially
rewritten by pitch review, and only the retrospective happens to
mention it.

## Sketch

Reuse the durable-comments container shape (idea 11) but let entries
cross-reference snippets of the codebase under review instead of other
archaeology artifacts. A feature branch that will become a PR mints a
review document hosting discussion threads and references into the
code as it stands on that branch; the code changes as a function of
the review, and before the branch enters the proper PR flow (final
approvals, pre-merge gates), the archaeology is stamped and included —
with the last round of changes distilling concrete decisions and
evidence into the record as a TLDR of what was actually agreed.
Agents are the natural operators: the ceremony is mechanical, so an
agent can be activated when a PR opens, run the ritual, and defer
approval to the humans. Open question inherited from decision 10:
sub-artifact fragments (`#` is reserved unused) would be needed to
address individual threads, and code references need a
line-or-snippet anchor form that survives the code changing under
review.

## Evidence

Henry's observation during sprint 5 pitch review, prompted by the
"pitch amended in review" pattern: sprint 4's task 13 and sprint 5's
task 17 were both reshaped by review conversation that the repository
does not durably hold. [[idea-comment-threads|Comment threads]] is
the container this would reuse; decision 10 reserved `#` for exactly
this fragment question. Contrast with PR comments as prior art: same
content, no durability, no cross-references, no archaeology.

## Incident evidence (2026-07-23)

Sprint 6's rationale named its review threads provisional test data
for this idea alongside idea 11, and the Sprint 5 post-merge review
(closed 2026-07-22) then ran a full post-merge variant of the
ceremony: [[cmt-sprint5-post-merge-stop-the-line|thread 3]] anchored
a `git-range` baseline, hosted adjudication threads bound to code
under review, and distilled the agreed outcome into decisions, tasks,
and a final disposition — the TLDR-into-the-record step this sketch
proposes, performed by hand. The pre-merge form this idea actually
proposes remains untested; the incident exercised the container and
the distillation, not the PR-flow timing. Status stays parked; the
managed-thread mechanics it would reuse are idea 11's promotion
question, not this idea's.
