---
id: ide_01KY5X7C56KBFWJJJKHTEXXQXV
sequence: 18
kind: idea
status: parked
created: 2026-07-22
---

# Modification watermark for scan performance

## Problem

If collections abandon lifecycle subdirectories for flat per-collection
directories (sprint 5's placement question), every status-filtered
query — "open dragons", "parked ideas", the most common operations —
must parse front matter for the whole collection, including an
ever-growing long tail of terminal-state artifacts. At today's scale
this is microseconds; a repository with thousands of closed artifacts
pays a linear scan for answers that concern only the active few.

## Sketch

A progressively updated watermark: a small generated file per
collection recording the modification timestamp of the oldest
non-terminal artifact. A scanner may skip any file older than the
watermark when it only needs non-terminal artifacts, because everything
past it is known-terminal. The watermark is a disposable projection in
the decision 1 sense — deletable at any time, rebuilt by one full scan,
never canonical, and `doctor` can verify it cheaply. Do not build it
until a real repository demonstrates a felt scan cost; the flat-layout
decision should only note the seam exists.

## Evidence

Raised by Henry while weighing flat placement for sprint 5: the known
cost of one flat directory per collection is that status filters read
everything, and this is the counter-lever if it ever hurts. Prior art:
mail dir readers and build systems skipping by mtime watermark.
[[idea-declarative-collection-specs|Declarative collection specs]]
would give the watermark one generic implementation point instead of
per-collection ones.

## Adjudication note (2026-07-22, comment thread 8)

Reviewed in [[cmt-s5-read-cost-and-watermark|thread 8]]; retained
parked with the claim narrowed, not rewritten:

- The watermark can only ever be a best-effort disposable hint, never
  correctness authority. A false skip is silent wrong output, and the
  invalidation failures are real: preserved-mtime copies (`cp -p`,
  `tar`, `rsync -t`), backdated edits (`touch -t`/`-r`), clock
  anomalies (a future-dated watermark skips everything), and
  same-second edits under coarse mtime granularity. Branch switches
  are mostly conservative in practice only because `git checkout`
  rewrites changed files with fresh mtimes — an emergent property of
  Git's implementation, not a contract.
- Verifying the premise ("everything older is still terminal")
  re-reads the front matter the skip hoped to avoid, so a validated
  watermark saves nothing and an unvalidated one is trusted blindly.
  Correctness must never consult it; deleting it may change
  performance only, never any command's output.
- Thread 8's cost mapping found the dominant scan cost today is bytes
  *retained* (every strict scan holds the full corpus in memory to
  print summaries), not bytes read — and the retention remedy is a
  summary-plus-locator read seam that needs no cache at all.

Prerequisites before un-parking: a felt scan cost in a real
repository (this idea's original bar); the summary/locator seam
first; and a demonstrated invalidation scheme.

## Incident-closure note (2026-07-23)

The Sprint 5 post-merge review closed on 2026-07-22 with
[[cmt-s5-read-cost-and-watermark|thread 8]] resolved
`accepted-deferred` — explicitly neither rejected nor silently
forgotten. This idea is the durable owner of that seam; the incident
closed legitimately with the item deferred, and deferred means owned
with a trigger, not vanished from active memory. Decision 13
additionally routed the catalog-aware-isolation variant (skip a
malformed sibling only when the identity catalog proves it is not a
claimant) through this idea's read-architecture prerequisites rather
than ad hoc read-policy changes.

Reactivation, restated from thread 8's adjudication without
embellishment:

- **Trigger:** a felt scan cost in a real repository — this idea's
  original bar, unchanged.
- **The pressure the seam addresses:** thread 8's cost mapping found
  bytes *retained* dominate, not bytes read — every strict surface
  holds the full corpus in memory where all summaries plus one
  payload would suffice, with doctor's double read as the standing
  aggravation. The retention remedy is the summary-plus-locator read
  seam, which needs no cache at all; the watermark remains a
  best-effort disposable hint, never correctness authority.
- **Real need versus speculative machinery:** a measured cost on a
  real corpus that distinguishes the competing designs. At
  adjudication scale (79 files, ~300 KB, 20 KB maximum) no probe can;
  until one does, building anything here is speculation.
- **Smallest step when the trigger fires:** design and measure the
  summary/locator seam first (thread 8's recorded prerequisite and
  the obligations it lists — identity re-validation at the lazy load,
  task 22's bound at the second read site, task 27's sibling policy);
  only afterward, and only with a demonstrated invalidation scheme,
  weigh the watermark itself.

Status stays `parked`; nothing above changes the claim.
