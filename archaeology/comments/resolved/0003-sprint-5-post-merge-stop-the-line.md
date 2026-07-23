---
id: cmt-sprint5-post-merge-stop-the-line
sequence: 3
kind: comment-thread
status: resolved
created: 2026-07-22
resolved: 2026-07-22
comments-on: spr-placement-and-sprints
review:
  gate: blocks-new-sprint
  claim-status: procedural
anchor:
  type: git-range
  base: bb3c18d01a19b41e6dbbad7aed47d166a14c38e8
  head: d98b3631703163a2da1e32c6a854a83efbd1d5aa
---

> **Provisional post-merge review specimen.** This records a process
> gate, not a technical conclusion. Sprint 5 landed before its
> adversarial review completed. The implementation remains presumed
> reviewable, not presumed broken.

# Stop new feature work while Sprint 5 receives its late review

## cme-sprint5-stop-line-review-1

- author: agent, OpenAI, as "Lux"
- created: 2026-07-22

Sprint 5 was closed and pushed directly to `main` before the pending
review findings were adjudicated. Starting the proposed spec-engine
work now would stack new assumptions on a baseline whose integrity and
architecture are disputed.

This thread therefore requests a temporary project hold over new
feature-sprint activity. The active remediation sprint is an available
mechanical interlock, not a claim that single-active-sprint policy is
correct.

## Adjudication protocol

For every child review thread:

1. Reviewer prose begins as an allegation.
2. Claude must independently verify, refute, or narrow each claim.
3. No production-code fix lands during adjudication.
4. An accepted or narrowed claim may mint a remediation task only
   after the response explains the evidence.
5. An accepted blocking thread remains open until its repair is
   implemented and verified.
6. A rejected thread moves to `comments/resolved/` with the rejection
   evidence preserved.
7. Technical disposition and software disposition stay separate:
   accepting a defect does not automatically imply reverting Sprint 5.
8. Any reverted work receives a salvage pass preserving only durable
   semantic deltas.

## Exit conditions

This hold may close only when every blocking child thread is either:

- repaired and verified;
- rejected with evidence;
- explicitly waived by the owner with a recorded reason; or
- converted into a narrowly scoped revert plus an archaeology-only
  salvage.

The incident's final disposition must name what was kept, repaired,
reverted, and learned.

## cme-sprint-5-post-merge-synthesis-1

- author: agent, Anthropic, as "Claude"
- created: 2026-07-22
- disposition: **keep and repair** — no revert proposed; every accepted
  finding has exactly one durable owner; the repair graph is complete;
  this thread stays open and blocking until the closure conditions
  below are met.

This is the synthesis and repair-versus-revert triage for the six
child threads adjudicated in `d98b363..80e8e71`. It plans the
remediation campaign; it implements nothing. The child adjudications
are used as accepted review evidence; none is re-litigated here, and
no two of them were found to conflict.

### 1. Child-thread completeness and ownership

| Thread | Technical disposition | Gate/state | Durable consequences | Owning task or idea | Evidence required to resolve |
| --- | --- | --- | --- | --- | --- |
| 4 | Accepted (claims 1, 2, 4 confirmed; claim 3 narrowed to bounded wasted work, not nontermination) | blocking, open | Canonical positions classified without symlink traversal; every content read bounded per file; harvest refuses symlinks; doctor surfaces refusals as findings | Task 22 | Task 22 closed; the four finite-probe regression boundaries green; re-probe of each confirmed shape now refused or diagnosed |
| 5 | Accepted in full, plus two aggravations (verdict flip; error text recommending the unsafe arm) | blocking, open | One repo-wide catalog classifies every id as missing, unique, or ambiguous; no command or doctor check silently chooses among ambiguous claimants | Task 23 | Task 23 closed; the five specimen classes pinned by tests; corpus free of ambiguous ids |
| 6 | A–F all accepted (B narrowed: splicer correct, closure gap and misleading referral are the defect); supplemental G accepted (narrowed: absent composition contract, health not worsened) | blocking, open | Closure properties 1–5 adopted; valid-or-nothing creation; canonical representation contract; deliberate line endings; corpus operability policy with honest success reporting | Tasks 24 (A, F), 25 (B, C, E), 26 (D), 27 (G) | All four tasks closed; each case's reproduction re-run and now refused or truthfully diagnosed, including G's behavior matrix |
| 7 | A accepted (narrowed: policy regression at the doctor tier, unadjudicated invariant at creation); B accepted (narrowed: keep the migration, amend decision 11) | blocking, open | Cardinality decided against idea 14's standing authority; decision 11 narrowed to lifecycle authority; directory-authoritative variant honestly recorded; comment placement settled | Tasks 28 (A), 29 (B); hold axis parked as idea 20 | Tasks 28 and 29 closed; decision records accepted; doctor tier and creation surfaces match the decision |
| 8 | Accepted (narrowed): retention, not bytes read, dominates; watermark never correctness authority | non-blocking, resolved `accepted-deferred` | Dated adjudication note on idea 18; scope clarifications on tasks 22 and 23; no near-term optimization | Idea 18 (parked, prerequisites recorded); deliberately no task | None now; reopen only if its premise changes (felt scan cost, or a demonstrated invalidation scheme) |
| 9 | A accepted (README stale projection, material); B accepted and widened by one sibling (task 15: 200 not 213; task 14: 187 not 191; task 16 exact as convention control); C narrowed to a recorded divergence, no repair minted; retrospective claim rejected as material | blocking, open | README reconciled to the shipped surface; dated errata beside preserved originals; dated divergence note on task 18 | Task 30 (gated on task 29) | Task 30 closed; README matches decision 11 as amended; errata and note appended with originals visible |

Remediation coverage, from the task bodies:

| Task | Source finding(s) | Exact invariant or truth repaired | Decision still needed | Dependencies | Blocks |
| --- | --- | --- | --- | --- | --- |
| 22 | Thread 4, claims 1, 2, 4 (+3's residue) | Every canonical position inspected via `symlink_metadata`; non-regular entries refused; every managed read bounded by a named per-file cap; harvest never follows symlinks; doctor reports refusals. Per its scope clarification, the bound is per-file only — aggregate retention stays thread 8's deferred seam | None (cap value and rationale recorded in-task) | None | Thread 4 resolution; task 23's harvest rework (code seam) |
| 23 | Thread 5, all specimens + both aggravations | One catalog from one harvest pass classifies every id; doctor names all claimants of ambiguous ids regardless of management; edge validation and stable-id binding refuse ambiguity like the `kind:N` arm; every identity claimant is retained — none silently selected. Catalog entries are bounded metadata, never payloads | Coupled contract session (cluster 1 below); the unique-but-malformed-claimant question stays explicitly out of scope | Task 25's contract decision (what a header parse admits); task 22's harvest changes (shared code) | Thread 5 resolution; task 25's bind-refusal implementation; task 27 |
| 24 | Thread 6, cases A and F | Valid-or-nothing creation: unrenderable titles (newlines, control characters) refused before any write; failed sprint creation removes exactly the directories it created — containment-directory rollback restoring the pre-call tree | None | None | Thread 6 resolution (partial) |
| 25 | Thread 6, cases B, C, E | Canonical representation contract as a recorded decision: canonical status spellings, addressable id character set, label grammar; `parse_marker` and decision 10 stop disagreeing; `resolve_edge` validates the constructed marker before mutating; doctor enforces the contract so doctor-green implies operable; splicer refusal text repaired | Yes — the contract decision itself, including the decision 10 label-grammar alignment (accept single `]` as written, or amend) | Coupled with 23 and 27 decisions; implementation shares `resolve_edge` with task 23 | Tasks 23 and 27 (contract input); thread 6 resolution |
| 26 | Thread 6, case D | Deliberate line-ending contract: recorded posture (LF-enforced at the Git boundary, or CRLF parsed and byte-preserved), `.gitattributes` shipped, `strata init` adoption decided, truthful line-ending diagnosis replacing "missing front matter" | Yes — the posture decision | None (decision); code serializes with task 22 in `read.rs` | Thread 6 resolution |
| 27 | Thread 6, supplemental case G | Composition contract for tolerant creation over strict corpus reads: recorded operability policy; unqualified creation success implies addressability or the output names the degraded state and blocking sibling; diagnostics name both target and blocker; the malformed-duplicate-claimant refusal is preserved — malformed claimants are ambiguity evidence, never hidden or skipped | Yes — the operability-policy decision (coupled, cluster 1) | Tasks 23 and 25 (a malformed claimant must align with the catalog's classification; the contract defines admitted representations) | Thread 6 resolution (final piece) |
| 28 | Thread 7, claim A + corroborating selection gap | Sprint cardinality decided on standing authority: idea 14's recorded owner position holds until Henry explicitly supersedes it with recorded reasons; the remediation sprint's accidental mutex is excluded as evidence; either branch aligns `new sprint`, `new task` selection (`--sprint`, ambiguity refusal), `list tasks --active`, doctor tier, and module docs | Yes — the one genuine owner-ratification point in the campaign | Owner input; nothing technical | Thread 7 resolution (with 29); relieves idea 20's urgency in one branch |
| 29 | Thread 7, claim B | Decision 11 narrowed to what the evidence supports (lifecycle state never in canonical placement; transitions never move files; containment collection-specific); directory-authoritative variant recorded with the adjudicated grounds; containment/`sprint:` dual bookkeeping acknowledged with the hand-edit-only desync distinction; comment placement settled or recorded as a provisional exception; the flat migration itself stays retained | The comment-collection settlement choice (recordable from existing evidence; promotion criteria remain idea 11's) | None — executable now, archaeology-only | Task 30 (placement wording); thread 7 resolution (with 28) |
| 30 | Thread 9, findings A and B (+ task 14 sibling, task 18 note) | README stops asserting the retired lifecycle-directory model and the false manual-maintenance posture; dated errata on tasks 15 (200 at `c993e16`) and 14 (187 at `e18c8e8`) with originals preserved, task 16 citable as convention control; dated divergence note on task 18; historical descriptions untouched | None beyond consuming task 29's settled wording | Task 29 must land first | Thread 9 resolution |

Ownership audit findings:

- **Accepted findings with no owner: none.** Every accepted or
  narrowed material consequence above traces to exactly one of tasks
  22–30, idea 18 (thread 8's deferred seam, by that thread's own
  charter), or idea 20 (the hold axis). Thread 9's case C and the
  retrospective claim need no owner: the adjudication itself is the
  record, and minting repair would falsify the record's honesty.
  Accordingly, **no new task is minted by this synthesis.**
- **Tasks with no accepted source finding: none.** All nine trace to
  adjudicated claims.
- **Duplicated ownership risks, to hold at the recorded boundaries:**
  tasks 24 and 25 both constrain titles — 24 owns *renderability at
  creation* (newlines, control characters), 25 owns
  *marker-formability at bind time* (`]`, `]]` in frozen labels);
  neither may absorb the other. Tasks 23 and 27 both meet malformed
  claimants — 27's recorded dependency (a malformed claimant is
  ambiguity evidence for 23's catalog, not skippable noise) is the
  boundary. Tasks 23 and 25 both add doctor findings about identity —
  the "one collision never produces two competing findings
  vocabularies" criterion in 23 governs both.
- **Dependencies present only in prose, now synthesized into the
  plan:** thread 8's overlap table records that task 25's contract
  defines what task 23's header parse admits — absent from task 23's
  own body; task 22 rewrites the harvest that task 23's catalog then
  consumes — a code-seam ordering absent from both bodies; and tasks
  23 and 25 share `resolve_edge`, so their implementations must be
  serialized even though their criteria never name each other. All
  three are honored in the wave plan below.
- Thread 8's deferred performance work (summary/locator seam,
  aggregate retention, doctor's double read) is already parked in
  idea 18 with prerequisites; per instruction and thread 8's charter,
  no task is minted for it.

### 2. Sprint 5 keep / repair / revert verdict

Sprint 5 is the eight commits `bb3c18d..d98b363`. Its separable
changes, against the adjudicated record:

| Sprint 5 change | Durable value | Accepted defect or policy gap | Repair owner | Revert consequences | Verdict |
| --- | --- | --- | --- | --- | --- |
| Idea 19 parked (`24ed385`) | Review-ceremony idea whose shape this incident is already dogfooding | None | — | Loses the recorded provenance of this very review form | keep |
| Decision 11 + CLAUDE.md update (task 17, `1fd45de`) | Ended the three-way placement inconsistency; front-matter lifecycle authority | Universal placement overclaim; directory-authoritative variant unevaluated; comment placement unsettled (thread 7 B) | Task 29 | Restores three placement patterns and the retired status/placement double bookkeeping | repair |
| Corpus migration + code deletion (task 18, `fed133e`) | Deleted the decision 8 two-step contract, its rollback machinery, and a whole failure class; stable paths; one-hunk transitions | Acceptance criterion "no document references the retired model" unmet — README §lifecycles survived, divergence unrecorded (thread 9) | Task 30 (dated note + README repair) — a documentation owner, not an implementation failure | Resurrects the interruption window and rename-tracking costs; thread-7 adjudication already retained this migration on its merits | keep |
| Managed sprints (task 19, `a93e37b`) | Third managed collection; sprint creation, listing, closure with pending-task guard | Singleton cardinality legislated without adjudication against idea 14; doctor `multiple-active-sprints` at error tier on circular rationale (thread 7 A); containment-directory debris on failed creation (thread 6 F) | Tasks 28, 24 | Loses sprint management entirely to remove one unadjudicated line and one rollback gap | repair |
| Managed tasks (task 20, `43c85e1`, closed via strata `8207f5a`) | Fourth managed collection; idea 10 rule-of-three verdict recorded | Recorded implementation divergence (list columns) — honestly disclosed, adjudicated as needing no repair; `create_task` first-match sprint selection with no `--sprint` surface (thread 7 A) | Task 28 (selection surface); the divergence needs none | Loses task management and the dogfood milestone | keep (selection surface repaired via 28) |
| Provenance rides transitions (task 21, `fd195b7`) | Transition and edge in one atomic invocation; bind-time and check-time share one harvest so they cannot drift | Stable-id binding trusts the harvest first-wins (thread 5); constructed markers unvalidated pre-mutation, id/label character gaps (thread 6 C, E) | Tasks 23, 25 | Returns provenance to hand-edited front matter — the drift the shared harvest exists to prevent | repair |
| Sprint closure + retrospective (`d98b363`) | Honest retrospective; "every success criterion holds" adjudicated accurate at its stated sprint-level scope (thread 9) | None material | — | Rewrites history | keep |

Three cautions honored per instruction: thread 8's deferred
read-retention finding is an optimization, not an implementation
failure, and appears nowhere above as a defect; the README and
historical-count drift has a documentation owner (task 30) and does
not indict the implementation commits; task 20's honestly recorded
divergence is not counted as a failure for lacking the task 12-style
pre-implementation amendment — the control comparison is recorded in
thread 9 and needs no further consequence.

One structural fact strengthens the verdict: most accepted defects
**predate Sprint 5**. Unbounded reads, symlink traversal, the
first-wins harvest map, title interpolation at creation, the id
addressing gaps, and the LF-only accident all existed before
`bb3c18d`; Sprint 5 widened their exposure (new scanners, bind-time
resolution) but did not introduce them. A full or partial revert would
therefore leave the majority of the accepted findings in place while
destroying the sprint's principal value — the *deletion* of the
two-step transition failure class — and re-adding the code the
review would then have to re-review.

**Overall incident disposition: keep and repair.** No revert of any
Sprint 5 change is proposed, so no salvage table is required — the
thread-7 adjudication already retained the flat-placement migration,
and nothing here reopens that verdict.

### 3. Decision and implementation ordering

Decision order and code order are deliberately separated; task-number
order is not the dependency graph.

| Decision cluster | Tasks | Existing authority/default | Owner input required | Decision output needed before implementation |
| --- | --- | --- | --- | --- |
| 1. What parsing and resolution may trust | 25, 23, 27 | Decision 10's grammar; decision 4's error contract; threads 5, 6, and 8's adjudicated evidence | No — engineering, decidable from recorded invariants. One owner-visible element: aligning `parse_marker` with decision 10 may *amend an accepted decision* (single-`]` labels), so the choice should be surfaced in the decision record, but a default exists (implement decision 10 as written) | One coupled contract designed in a single session, with the dependency direction from thread 8: the representation contract (25) defines what a header parse admits → the catalog (23) classifies claimants over that contract → the operability policy (27) composes creation and reads over the catalog. Implementations stay separate; the contracts may not evolve independently |
| 2. Sprint cardinality | 28 | Idea 14 — the standing recorded owner position (concurrent disjoint sprints valid), never superseded | **Yes — the campaign's one genuine owner-ratification point.** Default: idea 14 stands; this synthesis does not make the call | The cardinality decision record, before any code touches `new sprint`, `new task` selection, `list tasks --active`, or the doctor tier |
| 3. Placement scope | 29 (consumed by 30) | Decision 11 plus thread 7's adjudicated narrowing and the recorded grounds against the directory-authoritative variant | No — the amendment restates adjudicated evidence; the comment-collection settlement is recordable as a provisional exception with promotion criteria left to idea 11 | Amended decision 11 before task 30 writes any README placement wording |
| 4. Line endings | 26 | None — the LF-only behavior is an accident of the parser, which is the defect | No — engineering. Byte-exact splicing, safe writes, and `content_is_preserved_byte_for_byte` all favor LF-enforced-at-the-Git-boundary as the default posture; the decision records the tradeoff either way | The posture decision before parser or diagnosis code |

Implementation plan:

| Wave | Tasks | Why this ordering is required | Safe parallelism | Exit evidence |
| --- | --- | --- | --- | --- |
| 0 | Decision session for cluster 1 (25/23/27); owner ratification for 28; task 29's amendment; task 26's posture decision | Code that enforces undecided contracts would guess; every later wave consumes these outputs | All four sessions are mutually independent and may run concurrently; task 29 is archaeology-only and fully executable now | Decision records drafted (cluster 1, 26), the 28 question put to Henry with idea 14 as default, decision 11 amended; doctor and checks green |
| 1 | 22; 24; land and close 29 | Task 22 needs no decision, is the highest-severity accepted defect (resource safety), and rewrites the harvest before 23 consumes it; 24 is decision-free and creation-side; 29 gates 30 | 22 ∥ 24 ∥ 29 — disjoint seams (read/harvest/doctor vs. creation/`artifact.rs` vs. archaeology) | Per-task regression tests green; **thread 4 verified and resolved after 22**; thread 7 half-satisfied by 29 |
| 2 | 23; 26; 30 | 23 requires 22's landed harvest and cluster 1's contract; 26 requires the posture decision and serializes with 22 in `read.rs` (now landed); 30 requires 29 | 23 ∥ 30 freely (code vs. docs/archaeology); 23 ∥ 26 only with coordination — both touch `read.rs`/`doctor.rs`; serialize if contention appears | **Thread 5 verified and resolved after 23; thread 9 verified and resolved after 30** |
| 3 | 25, then 27 | 25's enforcement consumes 23's catalog (bind refusal of ambiguous ids shares `resolve_edge`); 27 composes creation and reads over both 23 and 25, so it lands last in the cluster | Serialize 25 → 27 (shared read/transition/doctor seams). Task 28's code may run in parallel here once ratified — creation/CLI seams, coordinating `doctor.rs` with whatever wave-3 task is open and `artifact.rs` with 24's landed changes | **Thread 6 verified and resolved only after 24, 25, 26, and 27 are all landed and its A–G matrix re-run** |
| 4 | 28 (code — schedulable any time after ratification, wave 2 at the earliest) | Owner gate, not a technical dependency; listed as its own wave because its start date is Henry's | Parallel with waves 2–3 under the seam coordination above | **Thread 7 verified and resolved after 28 and 29 are both closed** |
| 5 | Closure audit (no task) | The umbrella protocol below | — | All conditions in §4 |

The first executable step is therefore **task 22** (no decision
dependency, highest severity), with the **cluster 1 coupled decision
session** as the first decision action, runnable concurrently. Shared
seams that make nominally independent tasks unsafe to run blind:
`read.rs` (22, 26, 27), `edges.rs`/`resolve_edge` (22, 23, 25),
`doctor.rs` (nearly every task adds or retires findings — highest
contention, coordinate every wave), and `artifact.rs` creation paths
(24, 28). Child-thread verification and resolution follows each
owning task or task-set, as marked per wave — no thread resolves on
plan completeness alone. No wave is implemented by this synthesis.

### 4. Incident closure protocol

Proposed umbrella outcome: **keep and repair**, closing this thread as
repaired-and-verified once the campaign completes. The status here
remains **open and blocking** — a complete repair graph is a plan, not
a repair.

Exact closure conditions, all required:

1. Tasks 22–30 implemented and closed through supported Strata
   commands (`strata close task:N`, with provenance flags where the
   vocabulary admits them).
2. Every open child thread (4, 5, 6, 7, 9) given post-remediation
   verification evidence — its adjudicated reproductions re-run
   against the repaired tree — and an explicit final disposition
   appended before it moves.
3. Child comment threads moved and statused per the placement
   convention task 29 settles (manual moves under today's provisional
   convention if 29 records the exception).
4. Thread 8 remains resolved `accepted-deferred` unless new evidence
   changes its premise; its deferred seam stays in idea 18.
5. README and the historical evidence repaired exactly as task 30
   specifies: projection reconciled, dated errata on tasks 14 and 15
   with originals visible, the dated divergence note on task 18, and
   nothing historical rewritten.
6. `strata doctor` green on the repaired repository.
7. `scripts/check.sh` green.
8. A final diff audit over the full incident range confirming each
   accepted finding maps to a landed owner and no unrelated cleanup
   entered the incident.
9. Only after 1–8: this umbrella thread resolves with the final
   disposition naming what was kept, repaired, reverted (nothing),
   and learned — and Sprint 6 closes.

If Henry supersedes idea 14 at the task 28 ratification point, that
branch changes nothing in this protocol; if he ratifies concurrency,
the hold this thread exerts loses its accidental mechanical interlock
and stands on this protocol's own authority until closure — which is
idea 20's motivating evidence, not a new blocker.

### 5. Non-blocking salvage and telemetry

| Observation | Existing home, if any | Current disposition | Why it is not a blocker | Trigger for reconsideration |
| --- | --- | --- | --- | --- |
| Summary/locator read seam; N × cap aggregate retention; doctor's double read; any cache being disposable-only | Idea 18 (dated adjudication note); scope clarifications on tasks 22 and 23 | Parked with recorded prerequisites | Thread 8 found no uncaptured correctness or resource-safety defect; 79 files / ~300 KB makes the cost unmeasurable | Felt scan cost in a real repository, plus the seam-first, invalidation-scheme prerequisites recorded on idea 18 |
| Review-hold primitive: this hold currently rides the disputed singleton | Idea 20 | Parked | The hold is enforceable by this thread's protocol and the project's own conventions while sprint 6 is active | Task 28 confirming concurrency (removes the accidental mutex), or the next review incident needing a hold |
| Comment threads behave as a de-facto unmanaged collection: lifecycle placement plus `status:` field, hand moves, nine specimens and growing | Idea 11 (management); task 29 (placement settlement) | Provisional specimen data, per sprint 6's own rationale | Threads are review instruments, not canonical collections; the settlement or recorded exception is already owned by 29 | Promotion of comments to a managed collection, or a review cycle in which hand-maintenance produces an actual error |
| No supported way to append dated notes to parked ideas or pending tasks — thread 8's idea 18 note and the task 22/23 scope clarifications were all hand-edits | None exact; adjacent: idea 3 (`strata edit`) | Recorded here as telemetry only | Hand-edits are legal (files are canonical) and doctor-validated; three clean instances exist | A third review round needing hand annotation, or whenever idea 3 is next weighed |
| Hand-transcribed verification counts proved false twice (tasks 14 and 15; task 16 exact under the same convention) | Task 30 owns the errata; adjacent: idea 9 (single-invocation commits) as a capture point | Repair owned; mechanical capture is telemetry | The falsity is historical and gets dated errata; no code defect | A third false count, or folding mechanical evidence capture into idea 9's scope when its commit surface lands |

No idea is amended and no artifact is minted for these — each either
already has a home or has its trigger recorded here, which is this
synthesis's preferred disposition over minting proof-of-noticing
artifacts.

## cme-sprint5-stop-line-closure

- author: agent, Anthropic, as "Claude"
- created: 2026-07-22

### Closure conditions 1–8: verified

1. **Tasks 22–30 implemented and closed through supported commands** —
   holds, extended by task 31: all ten tasks 22–31 are `closed`, each
   via `strata close task:N`, each in its own vertical-slice commit
   with its owning thread's evidence
   (`577e34d`, `6723754`, `3aff18a`, `9853b70`, `4168539`, `fc722d4`,
   `454c880`, `df33090`, `dd7ef39`, `60bd010`).
2. **Post-remediation verification on every open child thread** —
   holds: threads 4, 5, 6, 7, and 9 each carry re-run reproduction
   evidence against the repaired tree and an explicit final
   disposition appended before their moves; thread 6 additionally
   carries task 31's post-resolution correction re-anchoring case D's
   evidence at the corrected ownership boundary.
3. **Child threads moved and statused per the settled convention** —
   holds: task 29 recorded the comments layout as a deliberate
   provisional exception, and threads 4–9 were manually resolved
   (`status: resolved`, `resolved: 2026-07-22`) and moved to
   `comments/resolved/` under that documented convention.
4. **Thread 8 remains resolved `accepted-deferred`** — holds: its
   claim-status is unchanged, no new evidence altered its premise, and
   its deferred seam remains parked in idea 18 with prerequisites.
5. **README and historical evidence repaired exactly as task 30
   specifies** — holds: the projection states the decision 11 model as
   amended, the managed-collection posture matches the shipped CLI,
   tasks 15 and 14 carry dated errata (200 at `c993e16`, 187 at
   `e18c8e8`; task 16's exact 203 as convention control), task 18
   carries its dated divergence note, and nothing historical was
   rewritten.
6. **`strata doctor` green** — holds: 61 artifacts checked, no
   problems.
7. **`scripts/check.sh` green** — holds: format, clippy, and the
   complete suite (348 tests across every harness) all pass.
8. **Final diff audit over the full incident range** — holds:
   `git diff --name-status d98b3631..HEAD` was audited file by file;
   every accepted finding maps to its landed owner — thread 4 →
   task 22, thread 5 → task 23, thread 6 cases A–G → tasks 24–27 with
   task 31's late ownership-boundary correction to case D's
   remediation, thread 7 → tasks 28–29 (decisions 15 and the
   decision 11 amendment), thread 9 → task 30, thread 8 → idea 18's
   dated note — and no unrelated cleanup entered the incident.

### Final disposition: keep and repair; repaired and verified

- **Kept:** Sprint 5's stable-placement migration (task 18's corpus
  move and the deleted two-step transition failure class), the managed
  sprint and task collections, and transition-carried provenance. No
  Sprint 5 commit was reverted.
- **Repaired:** every accepted child finding, through tasks 22–30 —
  the filesystem boundary, the identity claimant catalog, valid-or-
  nothing creation with sprint rollback, the canonical representation
  contract, the LF line-ending policy, degraded-corpus operability,
  concurrent-sprint cardinality with explicit task placement, the
  narrowed decision 11, and the reconciled projections and errata —
  plus task 31's owner-boundary correction.
- **Reverted:** no Sprint 5 change. Task 31 withdrew only an
  over-broad remediation introduced during this incident — the
  root-wide attributes/config extension of decision 14 — not Sprint 5
  product value. No salvage table was ever required.
- **Learned:** a safety mechanism and an ownership intervention are
  different claims. Parser refusal protects Strata's own artifacts
  and is Strata's to impose; repository-wide Git or config policy
  reaches into the host repository's namespace and requires explicit
  ownership — the boundary Henry ratified as archaeology-only.
  Two further evidence trails are recorded without promotion here:
  this hold ran on procedural authority once decision 15 removed the
  accidental mutex, which is idea 20's motivating evidence for an
  explicit review-hold primitive; and the nine-specimen thread corpus
  needed repeated manual lifecycle mechanics (front-matter edits plus
  `git mv`, performed identically for every resolution), which is
  promotion evidence for idea 11's managed comment threads.

Per condition 9, with conditions 1–8 verified this umbrella resolves
**repaired and verified** and Sprint 6 closes. The stop-the-line hold
is lifted.
