---
id: tsk_01KY6364DV39W0DZ3N0NF8GBGB
sequence: 25
kind: task
status: pending
sprint: spr_01KY61D615FAC8VVSTD7QXX1DW
created: 2026-07-22
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

## Result
