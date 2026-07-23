---
id: dec-lf-line-ending-policy
sequence: 14
kind: decision
status: accepted
created: 2026-07-22
---

# LF-only line endings, enforced at the Git boundary

## Context

Thread 6 case D ([[cmt-s5-operability-closure|operability closure]],
owed by [[tsk_01KY6364E105F7AWT7RAZ264WZ|task 26]]) reproduced an
ordinary failure: front-matter discovery requires literal LF
delimiters (`---\n`, `\n---\n`), the repository shipped no
`.gitattributes`, and CI runs only on Ubuntu. A Windows-default
checkout (`core.autocrlf=true`) rewrites every artifact to CRLF, after
which the entire corpus fails to parse as `malformed-artifact: missing
front matter` — a diagnosis that misdescribes the actual state and
offers no repair path. "Git-friendly" silently meant "Git-friendly on
LF platforms only", and nothing had decided that.

## Decision: LF is the only canonical line ending

LF is the sole canonical line ending for Strata Markdown artifacts and
for `.strata.toml`. This is a format rule of the canonical byte
contract, on the same footing as the front-matter delimiters
themselves.

Enforcement is layered:

- **Git boundary.** Root `.gitattributes` rules pin LF at checkout
  wherever Git is present:

  ```text
  *.md text eol=lf
  /.strata.toml text eol=lf
  ```

  `strata init` materializes the same template into new repositories,
  so a fresh repository is protected before its first artifact exists.
- **Parser backstop.** The shared reader refuses CRLF sequences and
  bare carriage returns instead of normalizing them. One shared
  LF-conformance check runs before front-matter delimiter discovery
  for every managed artifact and before `.strata.toml` TOML parsing.
  Git remains optional at the core: the parser enforces the format
  even where no Git executable or `.git` directory exists.

Refusal, not normalization, is deliberate: it protects byte-exact
splicing (transitions rewrite exactly one front-matter line and
preserve every other byte) and keeps
`content_is_preserved_byte_for_byte` unambiguous — there is exactly
one canonical byte sequence for any artifact, and no read or write
path ever has to reason about a second line-ending representation.

## Decision: the diagnosis names the actual cause

A rejected file is `malformed-artifact`, and its reason names line
endings truthfully: CRLF (or a bare carriage return, named distinctly)
against the LF-only policy, with repair guidance — convert the file to
LF and retain the repository's `.gitattributes` policy. The refusal
must never fall through to "missing front matter", and the original
file remains byte-identical. A CRLF `.strata.toml` may prevent
repository discovery entirely, but its direct error still names line
endings as the cause.

## Decision: init materializes but never merges

`strata init` creates the `.gitattributes` template safely and
no-clobber. A pre-existing regular `.gitattributes` is preserved
byte-for-byte — never parsed, merged, or replaced: Strata cannot
safely infer or merge arbitrary existing Git-attribute policies, so
for repositories carrying their own policy the parser's LF diagnosis
is the backstop. A non-regular object at the path is refused as an
artifact conflict. An initialized repository missing the file gains it
on the next `strata init`, and the created path appears in
`InitReport`. Decision 5's documented nontransactional
empty-directory boundary is unchanged.

## Alternatives rejected

**Accept and byte-preserve CRLF.** The parser would treat `---\r\n` as
a delimiter and every mutation would preserve the file's existing
convention. Rejected: it makes every splicer and safe-write path
line-ending-sensitive, doubles the representation states every
byte-level contract must cover (and every test matrix must cross), and
weakens the repository-wide canonical-byte contract — all for no
demonstrated benefit, since Git attributes already deliver LF working
trees on Windows for repositories that declare them.

**Silently normalize on read or write.** Rejected: a read that returns
different bytes than the file holds breaks the files-are-canonical
invariant, and a write that rewrites line endings is a mutation the
user never requested.

## Consequences

- The corpus stays single-representation; splicing, safe writes, and
  byte-preservation tests reason about exactly one format.
- Windows checkouts of attribute-carrying repositories work; a
  repository that defeats the attributes gets a truthful, repairable
  diagnosis instead of a misleading one.
- Task 26 deliberately excludes generic newline normalization,
  encoding repair, and editor configuration.

## Amendment: archaeology ownership boundary (2026-07-22)

Henry ratified "archaeology-only LF" on 2026-07-22: this decision owns
LF only within the `archaeology/` tree. The original text above is
preserved as historical evidence; its root-wide Markdown and
`.strata.toml` claims are superseded by this amendment
([[tsk_01KY6PHGTEX6FMCC9V3T599ZRV|task 31]]). The decision remains
accepted.

### The corrected contract

- LF remains the canonical byte format for Markdown beneath
  `archaeology/`. Artifact parsing still refuses CRLF and bare
  carriage returns before front-matter delimiter discovery, preserving
  the single-representation splicing contract; refused files stay
  byte-identical, and nothing is silently normalized.
- The Git convenience policy now lives at
  `archaeology/.gitattributes`, containing exactly:

  ```text
  *.md text eol=lf
  ```

  Because the attributes file is inside `archaeology/`, it governs
  archaeology Markdown without annexing root README files or other
  host-repository Markdown.
- Root Markdown is outside Strata's ownership, and a root
  `.gitattributes` belongs to the host repository: `strata init` must
  not create, inspect, merge, reject, replace, or delete one — even
  when its contents disagree with Strata's policy.
- `.strata.toml` is ordinary TOML configuration, not a splice-mutated
  Markdown artifact. It sits outside the LF-only artifact-byte
  contract and accepts whatever line endings the TOML parser accepts,
  including CRLF; invalid TOML keeps its ordinary truthful TOML
  diagnosis.
- Git remains optional: the attributes file is a convenience layer,
  and the artifact parser remains the correctness backstop.

### Rationale: safety mechanism versus ownership intervention

Parser refusal is required for safe artifact mutation — byte-exact
splicing cannot tolerate a second line-ending representation, so the
fail-closed check is Strata's to impose on Strata's files. Governing
unrelated Markdown or config bytes is a different claim: it is neither
necessary for that safety property nor within Strata's namespace. The
original root-wide `*.md` rule rewrote the checkout behavior of every
Markdown file in the host repository, and the config LF refusal
rejected configurations TOML itself defines as valid, for no
mutation-safety gain — Strata never splices `.strata.toml`.

### Migration boundary

Task 26's root `.gitattributes` existed only on this unmerged review
campaign, so this repository removes that known file explicitly in
task 31's commit. No generic code deletes an arbitrary root
`.gitattributes`: Strata cannot prove who created or modified such a
file, so existing root policies are wholly outside the init surface.
