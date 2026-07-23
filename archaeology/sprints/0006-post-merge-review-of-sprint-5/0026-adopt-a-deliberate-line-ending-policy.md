---
id: tsk_01KY6364E105F7AWT7RAZ264WZ
sequence: 26
kind: task
status: closed
sprint: spr_01KY61D615FAC8VVSTD7QXX1DW
created: 2026-07-22
closed: 2026-07-22
---

# Adopt a deliberate line-ending policy

## Objective

Close case D of comment thread 6. Front-matter discovery requires
literal LF delimiters (`---\n`, `\n---\n`), the repository ships no
`.gitattributes`, and CI runs only on Ubuntu. A Windows-default
checkout (`core.autocrlf=true`) rewrites every artifact to CRLF, after
which every artifact fails to parse — reproduced: one such checkout
turns the whole corpus into `malformed-artifact: missing front
matter`, a diagnosis that misdescribes the actual state and gives the
user no repair path. "Git-friendly" currently means "Git-friendly on
LF platforms only", and nothing decided that.

Closure property from the thread: supported checkout line endings are
parsed and byte-preserved deliberately (5).

## Acceptance criteria

- A recorded decision states the line-ending posture: either artifacts
  are LF-only and the repository enforces that at the Git boundary, or
  the parser accepts CRLF and every mutation byte-preserves it. The
  choice is recorded with its tradeoffs (byte-exact splicing, safe
  writes, and `content_is_preserved_byte_for_byte` semantics all touch
  it).
- A `.gitattributes` implementing the chosen posture for `*.md` and
  `.strata.toml` ships at the repository root, and `strata init` is
  considered as a place to materialize the same policy for new
  repositories (adopt or explicitly decline in the decision).
- When Strata encounters an artifact whose line endings the posture
  excludes, the diagnosis names line endings as the cause and the
  repair (not "missing front matter").
- Regression tests cover a CRLF artifact for whichever posture was
  chosen: parsed-and-preserved, or refused with the line-ending
  diagnosis; plus a doctor run over a CRLF corpus.
- `scripts/check.sh` and `strata doctor` are green at close.

## Result

Closed 2026-07-22. Case D is remediated by
[[dec-lf-line-ending-policy|decision 14]]: LF is the only canonical
line ending for Strata Markdown artifacts and `.strata.toml`, enforced
at the Git boundary and backstopped by the parser.

**Decision and tradeoffs.** The rejected alternative — accept and
byte-preserve CRLF — loses because it makes every splicer and
safe-write path line-ending-sensitive, doubles the representation
states every byte-level contract must cover, and weakens the
repository-wide canonical-byte contract for no demonstrated benefit.
Refusal keeps byte-exact splicing and
`content_is_preserved_byte_for_byte` unambiguous. Silent normalization
was separately rejected as an unrequested mutation.

**Git boundary.** Root `.gitattributes` now ships exactly:

```text
*.md text eol=lf
/.strata.toml text eol=lf
```

The present corpus was already LF, so adding the file produced no
normalization diffs.

**Init boundary.** `strata init` materializes the same template into
new repositories with the atomic no-clobber discipline (shared
`write_template`, the config still written last): a pre-existing
regular `.gitattributes` is preserved byte-for-byte and never parsed —
Strata cannot safely infer or merge arbitrary Git-attribute policies,
so the parser's LF diagnosis remains the backstop for such
repositories; a non-regular object at the path is an
`artifact-conflict`; an initialized repository missing the file gains
it on rerun; the created path appears in `InitReport`. Decision 5's
nontransactional empty-directory boundary is unchanged, and no Git
executable or `.git` directory is required.

**Shared check and diagnosis.** One shared check,
`read::lf_violation`, runs before front-matter delimiter discovery in
`parse_artifact_at` (every managed artifact, so `list`, `show`,
transitions, and doctor all inherit it) and before TOML parsing in
`repo::validate_config` (init and discovery). CRLF and bare CR are
each refused as `malformed-artifact` naming the actual cause, the
LF-only policy, and conversion-to-LF repair guidance; the refusal
never decays into "missing front matter" and never touches the file.
Doctor collects line-ending findings with correct paths; a CRLF
`.strata.toml` blocks discovery but its direct error names line
endings truthfully.

**Evidence.** Unit:
`crlf_artifact_is_refused_naming_line_endings_not_front_matter`,
`bare_carriage_return_is_diagnosed_distinctly_from_crlf`,
`lf_violation_accepts_lf_only_content`,
`validate_config_refuses_crlf_naming_line_endings_before_toml`,
`init_materializes_the_line_ending_policy`,
`existing_gitattributes_is_preserved_byte_for_byte_and_never_parsed`,
`gitattributes_path_occupied_by_directory_is_a_conflict`,
`initialized_repository_missing_gitattributes_gains_it_on_rerun`.
Integration (`tests/line_endings.rs`, `tests/init.rs`): CRLF refusal
by `show`, `list`, and `close` with byte-identical files; bare-CR
distinct diagnosis; doctor naming every CRLF path and cause; CRLF
config diagnosis; LF parse/transition byte preservation;
`shipped_gitattributes_carries_the_exact_two_policy_rules`; fresh-init
materialization without Git; idempotent rerun; preserved existing
policy; refused directory conflict. Complete suite 325 tests green
(204 lib + 121 integration); `strata doctor` 60 artifacts, no
problems; `scripts/check.sh` passes. Task 24 title/rollback and
task 25 representation regressions remain green.

## Supersession note (2026-07-22)

This task's Result accurately records what landed at commit `4168539`.
Its root-wide `.gitattributes` (covering all host-repository `*.md`
plus `/.strata.toml`) and the config LF refusal were subsequently
judged broader than Strata's ownership: Henry ratified
"archaeology-only LF" on 2026-07-22, and
[[tsk_01KY6PHGTEX6FMCC9V3T599ZRV|task 31]] with
[[dec-lf-line-ending-policy|decision 14]]'s amendment establish the
final boundary — the Git policy lives at
`archaeology/.gitattributes`, root Markdown and root `.gitattributes`
belong to the host repository, and `.strata.toml` accepts what the
TOML parser accepts. The artifact-side safety this task built —
fail-closed CRLF/bare-CR refusal before front-matter discovery, the
truthful line-ending diagnosis, byte-identical refused files — remains
intact and unweakened. The original acceptance criteria and Result
above are preserved unchanged.
