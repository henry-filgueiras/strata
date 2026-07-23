---
id: dec-degraded-corpus-operability
sequence: 13
kind: decision
status: accepted
created: 2026-07-22
---

# Degraded-corpus operability: documented degraded mode

## Context

Thread 6 case G ([[cmt-s5-operability-closure|operability closure]],
supplemental round) adjudicated the composition of two individually
deliberate boundaries: flat sequence allocation is filename-only and
content-blind, while `list`, `show`, and the transition resolvers
strictly parse every sibling and abort on the first malformed one.
Composed, `strata new dragon` beside a malformed sibling reports
unqualified success and prints a reference no read or transition
command can resolve until the unrelated sibling is repaired. Neither
boundary is defective; the missing piece is a documented contract for
their composition, owed by
[[tsk_01KY640RFXZJMWZ2T8W9B628AA|task 27]] as its first acceptance
criterion.

This decision consumes the representation contract recorded by
[[dec-canonical-representation|canonical representation and identity
addressability]]: what counts as an identity claim, an admitted
representation, or a malformed sibling below is what that record
defines, and the claimant catalog it governs (built by
[[tsk_01KY62E9VMB6HDNJWD31YS1FBP|task 23]], its admission threshold
adjudicated by thread 5,
[[cmt-s5-global-identity-catalog|global identity catalog]]) is the
identity surface every resolution path here trusts. The dependency
runs in that direction only.

## Decision: documented degraded mode

Among task 27's candidate policies, the chosen posture is the third:
**creation remains available in a documented degraded mode, and its
success reporting becomes honest about reachability.** Strict read
semantics are unchanged; what changes is that creation stops implying
an operability it cannot deliver.

## Decision: the flat creation contract

For flat creation (`new dragon`, `new idea` — artifacts written
directly into their collection directory):

- sequence allocation remains filename-only and content-blind; a
  malformed sibling occupies its sequence, sequences are never
  reused, and unrelated malformed content neither blocks allocation
  nor the write;
- after a successful write, an observational reachability check may
  attempt the corresponding strict command path; its first failure is
  by construction the deterministic blocking artifact;
- a failed reachability check never rolls back the already-successful
  creation and never changes the exit status: the write happened, and
  reporting it as failure would be false;
- the result distinguishes two shapes unambiguously: **created and
  reachable**, and **created, but repository degradation currently
  blocks normal access**;
- the degraded shape names both the created target and the
  deterministic blocking path or reason.

## Decision: the output contract

One stable, machine-visible warning classification serves human and
JSON invocations alike:

- the exit status remains 0 — creation succeeded;
- the normal success payload remains on stdout, and `--json` stdout
  remains valid and unpolluted;
- stderr carries a stable warning token line,
  `warning[degraded-repository]:`, naming the created target and the
  blocking path;
- the frozen `error[<code>]:` token is never used for this state —
  decision 4 ([[dec-bootstrap-error-contract|error contract]])
  reserves it for failures, and this is not one. The warning token
  follows decision 4's channel discipline (machine tokens and
  diagnostics on stderr, stdout parseable on every path) and becomes
  a compatibility surface of its own: renaming it is a breaking
  change requiring a new decision.

## Decision: the flat-versus-containment boundary

The tolerant contract above is exactly flat creation's. `new sprint`
and `new task` require strict containment scans before creating
anything — the active-sprint check and sprint resolution — and may
therefore fail on a malformed sibling without writing a file. This
divergence is deliberate and preserved, not silently unified:
containment creation resolves identity and lifecycle questions that
flat creation does not ask, and answering them from a corpus that
cannot be strictly scanned would mean guessing. Unifying the two
postures in either direction is out of scope of the accepted
contracts and would need its own decision.

## Decision: strict surfaces are unchanged

- Identity-dependent operations — provenance binding, typed-edge
  validation, stable-id resolution — consume task 23's complete
  claimant catalog; a malformed duplicate claimant remains ambiguity
  evidence, refused and named, never skippable noise.
- `list`, `show`, and the mutation resolvers keep their strict scans;
  no global permissive isolation of malformed siblings is
  introduced.
- When a command cannot reach an otherwise valid target because of a
  malformed sibling, the diagnostic names both the requested target
  and the blocking artifact, rather than only the sibling.
- Doctor remains the exhaustive collect-all diagnostic surface; the
  reachability probe above is observational and reports one blocker,
  never a substitute for doctor's full sweep.
- No global repository-valid bit is introduced anywhere: validity
  remains a per-question judgment (this scan, this claimant, this
  representation), not a repository flag.

## Decision: the composition invariant

A creation that reports unqualified success yields an artifact that
`show` (by sequence and by stable id), `list`, and its admitted
lifecycle transitions can reach. Otherwise the output explicitly
qualifies the degraded state and names the blocking sibling. Removing
only the blocker restores full reachability with no repair to the
created artifact itself — creation in degraded mode never worsens
repository health.

## Alternatives rejected

- **Refuse all creation in a degraded corpus.** Discards the
  adjudicated value of tolerant allocation: allocating past a
  malformed sibling is collision-safe by construction, is pinned by
  test, and lets work continue — including creating the very
  artifacts that record the repair. Case G's verdict was explicit
  that creation does not worsen health; refusing it buys nothing.
- **Globally isolate malformed siblings in ordinary reads.** Case G
  determination 6: the strict scan is currently the guard that
  surfaces a malformed duplicate claimant of a requested sequence or
  id. A skip-malformed read policy hides exactly the claimants the
  identity catalog exists to keep visible.
- **Catalog-aware isolation now.** The genuinely attractive future
  variant — skip a malformed sibling only when the catalog proves it
  is not a claimant of the requested identity — is deferred, not
  rejected on the merits: it belongs with the read-architecture and
  caching prerequisites parked with idea 18
  ([[ide_01KY5X7C56KBFWJJJKHTEXXQXV|modification watermark]]), and
  landing it before the catalog exists would be isolation policy
  built on the surface this review just convicted.

## Consequences

- Task 27 implements this policy: the reachability probe, the
  qualified success output, the warning token, and the
  target-and-blocker diagnostics, against the adjudicated case G
  behavior matrix.
- Thread 6's closure property 2 is honored in the honest form the
  supplemental round allowed: doctor-green implies operable per
  artifact, and where a red sibling breaks reachability, creation
  says so instead of overpromising.
- The `warning[degraded-repository]` token joins the stable machine
  surfaces; automation may branch on it without parsing prose.
- If strict-scan cost or the isolation question is ever felt again,
  the evidence trail runs through this record to idea 18's
  prerequisites rather than through ad hoc read-policy changes.
