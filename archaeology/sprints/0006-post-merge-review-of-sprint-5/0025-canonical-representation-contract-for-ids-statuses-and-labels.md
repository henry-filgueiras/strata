---
id: tsk_01KY6364DV39W0DZ3N0NF8GBGB
sequence: 25
kind: task
status: closed
sprint: spr_01KY61D615FAC8VVSTD7QXX1DW
created: 2026-07-22
closed: 2026-07-22
---

# Canonical representation contract for ids, statuses, and labels

## Objective

Close cases B, C, and E of comment thread 6. All three are one gap:
the read model admits representations that other surfaces —
transition splicing, CLI addressing, the marker grammar — refuse or,
worse, accept and then corrupt. Doctor validates only the read
model's side, so it blesses artifacts other operations cannot handle,
and successful mutations can leave the repository doctor-red.

Case B: YAML spellings such as `status: "open"` and `status: open #
note` deserialize to an admitted status and pass doctor, but the
status splicer deliberately (and correctly) rewrites only the plain
`status: open` form, so a doctor-green artifact is untransitionable.
The refusal even directs the user to `strata doctor`, which reports
nothing.

Case C: the read model documents ids as "any non-empty string", but
the CLI routes every `:`-bearing reference to the `kind:N` parser
(such ids cannot be shown or transitioned by id), and the decision 10
target grammar forbids whitespace, `#`, `|`, and `]`. Binding
provenance to a harvestable id containing such characters *succeeds*
and writes an unparseable marker: reproduced with `--resolved-by "dec
spacey"`, close exits 0, doctor then reports `invalid-edge`.

Case E: decision 10 says labels may contain anything but `]]` and
newlines, but `parse_marker` rejects any `]`; and `resolve_edge`
freezes the target's title into the label without validating the
constructed marker. A legal title such as `Handle the arr[0] edge
case` lets `close --resolved-by` succeed and makes doctor fail
afterward; a `]]`-bearing title does the same.

Closure properties from the thread: every doctor-green artifact is
operable for every action its state admits (2); a successful mutation
leaves the repository doctor-green (3).

## Acceptance criteria

- One recorded decision states the canonical representation contract:
  which status spellings are canonical, which characters an
  addressable id may contain (at minimum excluding `:`, whitespace,
  `#`, `|`, `]`, and newlines), and the label grammar `parse_marker`
  must implement. Where the contract narrows an existing documented
  promise (the "any non-empty string" id doc) the narrowing is
  explicit.
- `parse_marker` agrees with the decided label grammar. Either the
  code accepts a single `]` in labels per decision 10 as written, or
  decision 10 is amended; the two may not keep disagreeing.
- `resolve_edge` validates the marker it constructs before any
  mutation: a target whose id or frozen title cannot form a parseable
  marker is refused with a typed error naming the offending character
  class, and the source artifact is untouched.
- Doctor reports every representation the contract excludes on an
  otherwise parseable artifact — non-canonical status spellings,
  unaddressable ids — at a decided severity, so doctor-green implies
  operable. Existing hand-authored corpus ids (e.g. `drg-*`, `dec-*`)
  must remain conforming.
- The status splicer's refusal text no longer refers the user to
  doctor for states doctor accepts; it names the canonical spelling
  and, once doctor flags the state, the two surfaces agree.
- Creation and transition never write a representation the contract
  excludes.
- Regression tests cover: quoted and comment-bearing status spellings
  (doctor-flagged, splice-refused without corruption); a `:`-bearing
  id (unaddressable by id, diagnosed by doctor); provenance binding
  against a whitespace-bearing id and against `]`- and `]]`-bearing
  titles (refused pre-mutation, bytes untouched); and a single-`]`
  label round-tripping per the decided grammar.
- `scripts/check.sh` and `strata doctor` are green at close.

## Clarification (2026-07-22, decision 12)

The final regression bullet above predates decision 12
([[dec-canonical-representation|canonical representation]]) and
contradicts it in one clause: it asks that both `]`- and `]]`-bearing
titles be refused. Decision 12 took the recorded default — implement
decision 10's label grammar as written — so:

- a target title containing a single `]` is legal as a frozen label:
  binding it must succeed and the resulting marker must round-trip;
- a label containing `]]` or a newline is refused before mutation;
- this clarification supersedes only the stale single-`]` refusal
  phrase; every other criterion stands as written;
- decision 10 is not amended.

## Result

One addressability contract now exists: `edges::addressable` classifies
a decoded id against decision 12's excluded classes — empty, `:`,
Unicode whitespace (newlines included), `#`, `|`, `]` — returning a
typed `edges::IdViolation` so every diagnostic names the specific
offending class. Identity validity is untouched: any non-empty id
remains an identity, quoted and unquoted YAML spellings decode to the
same id (quoting alone draws no finding), excluded ids stay in task
23's claimant catalog as duplicate evidence, and addressability is
never a harvest filter
(`non_addressable_claimants_stay_catalogued_and_ambiguous`,
`unaddressable_duplicate_claimants_draw_both_findings`). The read
model's "any non-empty string" id documentation is narrowed explicitly
in `read::Summary` and `cli::ArtifactTarget`; no ULID structure is
required, and every current generated and hand-authored corpus shape
is pinned addressable (`every_current_id_shape_remains_addressable`).

Enforcement at the address and binding surfaces: the CLI refuses an
unaddressable bare id at parse time naming the class (`:`-bearing ids
were already captured by the `kind:N` grammar; both are pinned by
`unaddressable_bare_ids_are_refused_naming_the_class`), and
`transition::resolve_edge` validates the constructed semantic marker —
decoded id and frozen title — before any mutation: it checks the id
through `edges::addressable`, the title through `edges::label_valid`,
then round-trips `[[id|title]]` through `edges::parse_marker` and
requires the parse to return exactly the intended id and label; YAML
carrier escaping happens after and changes neither. Every refusal
leaves the source artifact byte-identical
(`binding_refuses_an_unaddressable_unique_target_before_mutation`,
`binding_refuses_a_double_bracket_title_before_mutation`, CLI twins in
`tests/provenance.rs`). Error category: refusals reuse the existing
`malformed-artifact` (exit 5) — chosen because the refusal's cause is
a property of the target artifact's canonical bytes rather than of the
invocation's shape, it is the category the bind path already used for
an unreadable title, and for unaddressable ids its "run `strata
doctor`" guidance is now truthful since doctor reports the same
artifact. No new frozen error code was added; `ambiguous-reference`
and `artifact-not-found` behavior is unchanged from task 23, including
the deferred unique malformed/probe-only target seam
(`unique_rejected_claimant_binding_preserves_the_deferred_seam` still
green).

Doctor enforcement uses decision 12's provisional
`non-canonical-artifact` vocabulary at error severity, distinct from
`malformed-artifact`, for two conditions: any admitted claimant —
managed, unmanaged/probe-only (the thread 6 case C hand-authored
decision shape), or canonically rejected alike — whose decoded id is
unaddressable (`unaddressable_claimant_ids_are_non_canonical_findings`
covers a quoted `:`-bearing managed id and a whitespace-bearing
unmanaged decision id); and any cleanly parsed managed artifact whose
mutable status carrier is noncanonical. Claimant disposition and
addressability stay separate: no claimant is reclassified, dropped, or
inferred from optional fields. Human and `--json` output share the
classification, severity, paths, details, and path-sorted ordering
(`human_and_json_output_agree_on_representation_findings`); decisions
remain unmanaged and `artifacts_checked` is unchanged; the real corpus
gains no finding (doctor: 60 artifacts, no problems).

One shared status-carrier recognizer,
`transition::canonical_status_carrier`, now serves both the transition
splicer and doctor: within the front matter only, exactly one line
beginning `status:` at column zero whose trimmed remainder is exactly
the admitted lowercase status; surrounding whitespace is accepted and
byte-preserved by the splice; quoting, inline comments, duplicate
top-level carriers, and any other spelling are refused, and indented
spellings are not carriers at all
(`duplicate_and_indented_status_carriers_are_refused_or_ignored`,
`rewrite_status_preserves_unusual_spacing_around_the_value`,
`accepted_whitespace_around_a_status_value_is_not_a_finding`). The
recognizer returns the semantic value's byte range, so the splicer
rewrites exactly the value. Canonical parsing stays distinct from
conformance: quoted and comment-bearing statuses still deserialize
(`list` works, the artifact is checked), doctor reports them as
`non-canonical-artifact`, transitions refuse them before writing, and
the refusal now names the canonical spelling (`status: <word>`) so its
doctor referral is truthful — the thread 6 case B contradiction is
closed
(`quoted_status_parses_semantically_but_is_non_canonical_and_untransitionable`).
Strict reads were not turned into representation enforcement;
line-ending posture remains task 26's.

`edges::parse_marker` now implements decisions 10 and 12 exactly:
anchored on the final closing `]]`, label from the first `|` to that
anchor, labels may contain whitespace, `#`, `|`, and single `]`s (a
label ending in one `]` yields raw text ending `]]]` and round-trips)
but not `]]` or newlines; bound targets obey the addressability
contract; `kind:N` sugar semantics, including optional labels, are
unchanged (`parse_marker_accepts_single_brackets_in_labels_and_round_trips`,
`parse_marker_rejects_bracket_bearing_targets`). The single-`]`
clarification above was implemented as written: binding to a
single-`]` title succeeds and leaves doctor green
(`binding_freezes_a_single_bracket_title_that_round_trips`,
`binding_to_a_single_bracket_title_succeeds_and_doctor_stays_green`);
the stale refusal phrase was not implemented. No prose scanner was
anticipated: marker scanning in prose remains unbuilt per decision
12's recorded caveat.

Boundaries preserved: creation-title validation and sprint rollback
untouched (task 24); line endings and `.gitattributes` untouched
(task 26); degraded-corpus creation output and sibling isolation
untouched (task 27); no claimant-catalog filtering, no
identity-admission change, no caching or repository-valid bit;
decisions and comments remain unmanaged. Creation and successful
transitions continue to emit only canonical representations (existing
template and splice pins, plus the doctor-green assertions above).

Regression evidence: 21 focused tests added (16 unit, 5 integration);
suite now 189 unit + 107 integration tests, all green with
`scripts/check.sh` and repository doctor (60 artifacts, no problems).
The task 23 ambiguous-claimant and unique-malformed-target
regressions remain green unchanged.
