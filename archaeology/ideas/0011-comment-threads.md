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

## Specimen findings (2026-07-21)

The motivating critique was run end-to-end as a provisional specimen:
`archaeology/comments/resolved/0001-transition-crash-contract.md`
(`cmt-transition-crash-contract`), anchored to
`tsk-lifecycle-transitions`, resolved with its conclusions promoted to
decision 8, dragon 4, and sprint 2 amendments. Format lessons, none of
which adopt the idea:

- Per-entry metadata as nested `---` fenced blocks inside the body
  collides with front-matter parsing conventions; a heading per entry
  plus a plain key list worked and stays grep-able.
- A `comments-on`/`target`/`relation` triple is redundant: decision 6
  already makes typed edges front-matter fields, so a single
  `comments-on: <stable-id>` suffices. The anchor is a separate
  structure because it refines *where*, not *what*.
- Quote anchors need whitespace normalization: Markdown hard-wrapping
  means the exact quote spans a line break in the source, so exact
  matching must be whitespace-insensitive or anchors break on rewrap.
- The anchor quote appeared verbatim in both `sprint.md` and task 7 —
  a quote selector alone does not identify the target; the typed edge
  disambiguates and the prefix/suffix mattered on first use.
- Successful resolution orphans the anchor by construction: promotion
  rewrites the quoted text. Orphaned-after-resolution is the *expected*
  terminal anchor state, which validates recording the optional
  `git-blob` historical referent.
- Resolution wants machine-legible structure (`disposition`,
  `promoted-to`), not just prose — otherwise the conclusion is buried
  in conversation, the exact anti-pattern this idea guards against.
- Agent authorship needed provider/model/label; the label ("Lux",
  "Claude") is decoration, echoing decision 6's frozen-label rule.
- Threads gravitated to the standard lifecycle-directory convention
  (`resolved/` holding `status: resolved`), more weight for declarative
  collection specs (idea 10).

## Open representation questions (2026-07-21)

Raised in discussion of the specimen; none are settled by it.

- **Entry identity.** Entries need addressable ids in the reference
  model regardless of representation: the sprint 2 amendment wanted to
  cite one entry and could only say "comments 1" in prose. The
  heading-per-entry form already yields tooling-free URL fragments
  (`0001-thread.md#cme-entry-id` renders on GitHub). Whether reference
  targets admit sub-artifact fragments, and the entry-heading grammar
  itself, are promoted into dragon 3's question (update recorded
  there).
- **Canonical form.** Three candidates, in tension between the thread's
  row-oriented structure and its narrative payload:
  1. one Markdown file per thread with hardened entry headings — what
     the specimen did; narrative wins, structure is convention;
  2. JSONL rows with Strata pretty-printing the parent plus threads —
     the honest structural fit for append-only records, and it would be
     the first non-Markdown collection, finally exercising decision 3's
     format-independence invariant; but multi-paragraph review prose as
     escaped JSON strings fails the raw-PR-diff readability test
     (decision 6's own acceptance criterion, and the not-hostage
     invariant in miniature) in exactly the context where review
     discussion is read;
  3. entry-per-file, thread-as-directory — the maildir-versus-mbox
     tradeoff: concurrent commenters create distinct files, so merges
     need no tail resolution at all; entries get file-grain identity;
     cost is many small files plus explicit ordering metadata. Most
     attractive if concurrent multi-agent commenting is the real
     workload.
- **Canonical/projection direction.** The Markdown and JSONL forms are
  mechanically derivable from each other, so the choice is which is
  canonical. The not-hostage test favors the human-readable form as
  canonical with a structured JSONL projection derived from it —
  inverting the pretty-print direction — since derived projections are
  cheap and disposable by decision 1.

## Second specimen findings (2026-07-21)

A second thread was run end-to-end as a deliberately different case:
`archaeology/comments/resolved/0002-fortune-reproducibility.md`
(`cmt-fortune-reproducibility`), an advisory product-design question
against `tsk-strata-fortune`, resolved with most of its proposal
rejected. New lessons, beyond the first specimen's:

- Single-label dispositions are too coarse. Both specimens strained
  the enum (`accepted-with-refinement`, `accepted-in-part`): a
  multi-point review resolves per point, so structured resolution
  wants accepted/rejected/deferred groupings, not one verdict.
- Deferral does not imply an artifact. The provisional template
  presumed `deferred` promotes to a parked idea; the specimen deferred
  two designs with no idea minted, because parking them would record
  speculative features with no action attached. `promoted-to` must be
  optional for non-accepted dispositions.
- Rejection residue is only grep-discoverable. The promotion rule
  governs accepted conclusions; a mostly-rejected thread legitimately
  leaves its rationale nowhere but the resolved thread. That is
  consistent with never-load-bearing, but it means "settled for now"
  tradeoffs can be reopened by someone who never finds the thread.
  Mitigation used: record the exclusion as an amendment on the target
  artifact. The general shape — surfacing resolved threads when a
  proposal approaches the same ground — is idea 12's problem.
- Anchor rungs have different orphaning profiles. The first specimen's
  quote anchor was orphaned by its own successful resolution; this
  thread's heading anchor survived the amendment made beneath it,
  since resolution edited under the heading rather than renaming it.
  Coarser rungs live longer; `git-blob` matters less as the rung
  coarsens. This validates the degradation ladder's premise.
- Advisory-ness needed no severity field. The never-load-bearing rule
  already encodes non-blocking; "should not block Sprint 2" lived in
  prose and cost nothing. No `severity`/`blocking` front matter earned
  its way in.

## Incident evidence: the Sprint 5 post-merge review (2026-07-23)

The Sprint 5 post-merge review
([[cmt-sprint5-post-merge-stop-the-line|thread 3]],
[[spr_01KY61D615FAC8VVSTD7QXX1DW|sprint 6]], closed 2026-07-22) ran
seven further threads through their whole lifecycle. The original
proposal above is unchanged; this section records what that campaign
demonstrated.

- Threads carried the incident's deliberation, and only its
  deliberation. Discussion ≠ conclusion held throughout: every
  accepted conclusion was still promoted into decisions (12–15 and
  the decision 11 amendment), tasks (22–31), or documentation, with
  the thread as provenance — exactly the promotion rule this idea's
  sketch states.
- Every resolution repeated the identical manual lifecycle operation:
  edit the thread's front matter (`status: resolved`, add the
  `resolved:` date) and `git mv` the file from `comments/open/` to
  `comments/resolved/`. Thread 3's closure and sprint 6's
  retrospective both record this was performed identically for every
  resolution; by incident closure the corpus held nine resolved
  specimens (roughly ten performances of the same by-hand mechanics
  in the repository's life so far). This recurrence is concrete
  promotion evidence, not aesthetic dislike of hand work.
- Decision 11's amendment made promotion of this idea the explicit
  trigger for settling canonical stable placement for managed
  comments. Two constraints recorded there bind any implementation:
  a managed transition must not preserve lifecycle-directory movement
  without a new explicit decision, and the current
  directory-plus-`status:` duplication is a tolerated provisional
  exception — not precedent for other managed collections.

Design pressures the incident sharpened, deliberately left
unresolved:

- one thread-level disposition may be too coarse (already seen in the
  second specimen; thread 3's children needed per-claim verdicts like
  "accepted (narrowed)" and per-case ownership);
- deferral is not identical to parking an idea (thread 8 resolved
  `accepted-deferred` with its seam owned by an existing idea; no new
  artifact was minted);
- entry identity and fine-grained citation may eventually matter (the
  incident cited individual entries and per-case determinations by
  convention only);
- provenance must survive promotion into canonical artifacts — the
  incident's decisions and tasks cite their originating threads, and
  a managed lifecycle must not break those trails;
- this is not a forum product: notifications, reactions, access
  control, synchronization, and similar machinery remain out of scope
  unless separately justified.

**Present disposition:** parked. Strengthened evidence is not
adoption; promotion remains an explicit decision.

**Promotion trigger / next investigation:** design the minimum
managed-comment lifecycle and stable-placement model; test whether
`new`, `list`, `show`, `resolve`, and possibly `reopen` are
sufficient verbs; decide entry identity and promotion/provenance
behavior before any implementation.
