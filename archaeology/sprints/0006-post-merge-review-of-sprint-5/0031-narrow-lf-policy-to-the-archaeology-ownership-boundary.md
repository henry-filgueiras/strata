---
id: tsk_01KY6PHGTEX6FMCC9V3T599ZRV
sequence: 31
kind: task
status: closed
sprint: spr_01KY61D615FAC8VVSTD7QXX1DW
created: 2026-07-22
closed: 2026-07-22
---

# Narrow LF policy to the archaeology ownership boundary

## Objective

Correct [[dec-lf-line-ending-policy|decision 14]]'s ownership
boundary. Task 26's remediation of thread 6 case D was broader than
Strata's namespace: it shipped a root `.gitattributes` claiming every
`*.md` in the host repository plus `/.strata.toml`, and made config
validation refuse CRLF although `.strata.toml` is ordinary TOML
configuration, not a splice-mutated Markdown artifact. Henry ratified
the corrected policy on 2026-07-22: decision 14 owns LF only within
the `archaeology/` tree. Parser refusal is required for safe artifact
mutation; governing unrelated Markdown or config bytes is neither
necessary nor within Strata's ownership.

## Acceptance criteria

- Decision 14 carries a dated amendment recording the owner
  ratification: Markdown beneath `archaeology/` is LF-only; the Git
  convenience policy lives at `archaeology/.gitattributes` with the
  single rule `*.md text eol=lf`; root Markdown and root
  `.gitattributes` belong to the host repository; `.strata.toml`
  accepts whatever line endings the TOML parser accepts; managed
  artifacts remain fail-closed on CRLF and bare CR before front-matter
  parsing; no silent normalization; Git remains optional. The
  original text is preserved as history and the migration boundary is
  recorded: this repository's task 26 root file existed only on this
  unmerged review campaign and is removed explicitly, with no generic
  root-`.gitattributes` deletion code.
- This repository ships the policy only at
  `archaeology/.gitattributes`; the task 26 root `.gitattributes` is
  deleted, with no repository-wide normalization diffs.
- `strata init` materializes `archaeology/.gitattributes` (not a root
  policy) with the existing atomic no-clobber discipline: created
  path in `InitReport`, idempotent rerun, pre-existing regular file
  byte-preserved and never parsed, non-regular object refused as an
  artifact conflict, missing nested policy regained on rerun, root
  `.gitattributes` never created, inspected, merged, rejected,
  replaced, or deleted; no Git required; decision 5's boundary
  unchanged.
- Config validation accepts CRLF: `version = 1\r\n` is valid,
  discovery and normal commands work through a CRLF config, init
  byte-preserves an existing valid CRLF config, and invalid TOML
  keeps its truthful TOML diagnosis. No config rewrite.
- Artifact behavior is unweakened: CRLF and bare CR remain refused as
  `malformed-artifact` before front-matter discovery with
  conversion-to-LF guidance naming the archaeology policy; refused
  files stay byte-identical; strict `show`/`list`/transition/doctor
  behavior and LF byte-exactness are unchanged.
- Task 26 carries a dated supersession note and resolved thread 6
  carries a post-resolution correction (status, date, and path
  preserved; case D remains accepted and remediated at the corrected
  boundary); contradictory task 26 tests are replaced rather than
  left asserting the superseded root/config policy.
- `scripts/check.sh` and `strata doctor` (61 artifacts) are green at
  close.

## Result

Closed 2026-07-22. Decision 14's ownership boundary is corrected per
Henry's 2026-07-22 ratification: LF is owned only within the
`archaeology/` tree.

**Decision amendment.** Decision 14 carries the dated
"archaeology ownership boundary" amendment: original text preserved as
history, root-wide Markdown and `.strata.toml` claims superseded, the
decision still accepted (no decision 16). It records the corrected
contract, the safety-versus-ownership rationale (parser refusal is
required for safe splicing; governing unrelated bytes is neither
necessary nor Strata's namespace), and the migration boundary.

**Ownership boundary.** Markdown beneath `archaeology/` is LF-only;
the Git convenience policy is `archaeology/.gitattributes` with
exactly `*.md text eol=lf`; root Markdown and root `.gitattributes`
belong to the host repository; `.strata.toml` accepts whatever the
TOML parser accepts; Git remains optional with the parser as the
correctness backstop; nothing is silently normalized.

**Nested attributes behavior.** This repository's task 26 root
`.gitattributes` is deleted explicitly (it existed only on this
unmerged review campaign) and the policy ships at
`archaeology/.gitattributes` with no normalization diffs. `strata
init` materializes the nested file with the existing atomic
no-clobber discipline: root-relative path in `InitReport`, idempotent
rerun, regained when missing, pre-existing regular file byte-preserved
and never parsed, non-regular object refused as `artifact-conflict`
(a non-directory parent component defers to the required-directory
walk's truthful conflict), root `.gitattributes` ignored and untouched
even when it disagrees, no Git required, decision 5's boundary
unchanged. No generic root-`.gitattributes` deletion code exists:
Strata cannot prove who created or modified such a file.

**Config behavior.** The decision 14 LF check is removed from
`validate_config`: `version = 1\r\n` is valid, discovery and normal
commands work through a CRLF config, init byte-preserves an existing
valid CRLF config, invalid TOML keeps its ordinary truthful diagnosis,
and no config rewrite exists. The shared `lf_violation` helper keeps
its name; its documentation and callers no longer claim `.strata.toml`.

**Artifact safety preserved.** CRLF is still `malformed-artifact` and
bare CR still distinctly named, checked before front-matter delimiter
discovery, with conversion-to-LF guidance now naming
`archaeology/.gitattributes`; refused files stay byte-identical, and
strict `show`/`list`/transition/doctor behavior and LF byte-exactness
are unchanged.

**Archaeology corrections.** Task 26 carries a dated supersession note
(its Result accurately records `4168539`; the boundary, not the
safety, was corrected). Resolved thread 6 carries a post-resolution
correction — status, `resolved:` date, and path untouched — recording
the ratification, correcting case D's remediation to the nested
policy, withdrawing the CRLF-config tests from the closure evidence
while retaining the artifact CRLF/bare-CR and doctor evidence, and
confirming case D remains accepted and remediated at the corrected
boundary. Contradictory task 26 tests were replaced
(`shipped_policy_exists_only_at_the_nested_archaeology_path`,
`validate_config_accepts_crlf_line_endings`,
`crlf_config_is_valid_and_discovery_succeeds_through_it`,
`invalid_crlf_toml_keeps_the_ordinary_truthful_toml_diagnosis`); none
asserts the superseded root/config policy.

**Evidence.** Focused: the init suite
(`init_materializes_the_nested_line_ending_policy_only`,
`root_gitattributes_is_ignored_and_untouched_even_when_it_disagrees`,
`existing_gitattributes_is_preserved_byte_for_byte_and_never_parsed`,
`gitattributes_path_occupied_by_directory_is_a_conflict`,
`initialized_repository_missing_gitattributes_gains_it_on_rerun`,
`init_preserves_an_existing_valid_crlf_config_byte_for_byte`, and the
integration equivalents in `tests/init.rs`), the config-acceptance
tests above, and the retained artifact matrix in
`tests/line_endings.rs`. Complete suite 348 tests green; `strata
doctor` 61 artifacts, no problems; `scripts/check.sh` passes;
tasks 24–30 regressions remain green.
