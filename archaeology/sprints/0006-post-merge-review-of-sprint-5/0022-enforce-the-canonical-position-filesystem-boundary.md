---
id: tsk_01KY61X8H0QY2FZN4JSNP519A8
sequence: 22
kind: task
status: closed
sprint: spr_01KY61D615FAC8VVSTD7QXX1DW
created: 2026-07-22
closed: 2026-07-22
---

# Enforce the canonical-position filesystem boundary

## Objective

Close the hostile-repository read boundary adjudicated in comment
thread 4. Strata's managed read paths inspect entries with
`Path::is_dir()`/`Path::is_file()` (both follow symlinks) and then
`fs::read_to_string` with no size bound, and the unmanaged edge harvest
follows directory symlinks while walking `archaeology/`. A Git-backed
hostile working tree — Git round-trips symlinks as mode-120000 blobs —
can therefore make an artifact position resolve to content outside the
repository root, to an unbounded device, or to an external archaeology
tree whose ids then satisfy repository-local provenance edges. The fix
extends the boundary discipline `repo.rs` already applies to the config
marker and required directories (inspect with `symlink_metadata`, refuse
non-regular entries explicitly) to every canonical artifact position,
and gives every content read a declared bound. This is the boundary
`SECURITY.md` already promises; the task makes the code honor it.

Adjudication (thread 4) accepted three claims and narrowed one:

- outside file content entering the read universe via a file-symlink at
  a managed artifact position — confirmed;
- unbounded reads exhausting memory (a symlink to `/dev/zero`, or simply
  an oversized regular artifact — no symlink needed) — confirmed;
- external identities satisfying provenance edges via a directory
  symlink under `archaeology/` — confirmed through the unmanaged
  harvest;
- directory-symlink loops causing nontermination — narrowed to bounded
  wasted work: the kernel's symlink-resolution limit already forces
  `read_dir` to fail, so walks terminate, but not following symlinks
  removes the tail and makes a visited-set unnecessary.

## Acceptance criteria

- Every canonical artifact position is classified with
  `fs::symlink_metadata` (no traversal). A symlink or other non-regular
  entry where an artifact file belongs is refused with a typed error
  naming the path and the invariant — never read, never followed. This
  covers the flat scanners (dragons, ideas), the sprint and task
  scanners, and `doctor`'s parallel per-file walk, so scanning and
  diagnosis stay in agreement.
- Every managed content read is bounded: reading an artifact larger than
  a declared limit is a typed `malformed-artifact`-class failure naming
  the path and the cap, not an unbounded slurp into memory. The bound is
  a named constant with a recorded rationale, applied uniformly wherever
  artifact bytes are read (`read`, `doctor`, and the harvest).
- The unmanaged edge harvest (`edges::harvest`) does not follow symlinks:
  entries are classified with `symlink_metadata`, symlinked directories
  are not descended, and symlinked files are not read. An external
  identity reachable only through a directory symlink under
  `archaeology/` does not enter the verification universe, so an edge
  relying on it is reported `dangling-edge` exactly as if the symlink
  were absent.
- `doctor` reports the refused positions as findings rather than
  aborting the run: a hostile entry is corruption to surface, not an
  environmental error that hides the rest of the report.
- Regression tests, using only safe finite temporary-repository probes,
  prove each boundary: (1) a file-symlink artifact in a managed
  directory is refused and its outside content never appears in any
  projection; (2) an oversized artifact yields the bounded-read error,
  not an OOM; (3) an id reachable only via a directory symlink under
  `archaeology/` is absent from the harvest universe and its dependent
  edge dangles; (4) a directory symlink at a managed artifact position
  remains an `artifact-conflict`. No test reads `/dev/zero` or opens an
  unbounded loop.
- `scripts/check.sh` and `strata doctor` are green at close. The bounded
  read must not regress the legitimate corpus: the existing repository's
  largest artifact stays well under the cap.

## Scope clarification (2026-07-22, comment thread 8)

The read bound above is per-file by design and must not be described
as bounding aggregate retention: after it lands, a strict scan may
still retain up to N × cap across N artifacts. Aggregate retention is
the deferred read-seam concern adjudicated in
[[cmt-s5-read-cost-and-watermark|thread 8]] (summary-plus-locator
scanning; recorded on idea 18), not this task's scope. If a lazy
payload load is ever introduced, the bound and the symlink refusal
apply at that second read site too.

## Result

One bounded-read seam now fronts every production content read:
`read::read_capped` pulls at most `MAX_ARTIFACT_BYTES + 1` bytes through
`File::take`, and `read::read_artifact_bytes` wraps it with a
`symlink_metadata` regular-file check, the oversized refusal, and the
preserved invalid-UTF-8 distinction. `MAX_ARTIFACT_BYTES` is 1 MiB
(1,048,576 bytes), recorded with its rationale on the constant: ~40× the
largest real artifact, small enough that one hostile file cannot exhaust
memory. The bounded `take` — not any metadata inspection — is the
enforcement, so the cap holds even when a file grows after being
classified, and an oversized payload is never fully allocated. A file
exactly at the cap parses; one byte over is a typed `malformed-artifact`
naming the path and the cap, surfaced identically to human and `--json`
callers (errors are stderr-only) and as a doctor finding rather than an
abort.

Every canonical position is classified without following symlinks: the
flat scanners and `read::managed_entries` (which also classifies the
managed directory itself), the sprint containment and `sprint.md`
checks, the task scanners, doctor's parallel walks (refusals become
`artifact-conflict` findings; the run continues), and
`edges::harvest`, which no longer descends symlinked directories or
reads symlinked files — an identity reachable only through a link stays
out of the verification universe, so the thread's forged-provenance
shape now reports `dangling-edge`. Not following links also removes the
loop tail: no visited-set or canonicalization is needed, and no claim of
race-free confinement against concurrent replacement is made. The
audit of production reads also routed the `.strata.toml` marker read
through the same capped primitive; the one intentional exclusion is
sequence allocation (`max_sequence_in`), which reads filenames only —
zero content bytes — and stays content-blind per the recorded thread 6
case G seam owned by task 27.

Affected command surfaces: every artifact-reading command inherits the
seam — `list`, `show`, `new` (scan-based sequence allocation for
sprints/tasks), `close`/`reopen`/`adopt`/`reject` (resolution and
bind-time harvest), `fortune`, and `doctor`.

Regression coverage (all finite; symlink construction unix-gated):
exact-cap acceptance, one-byte-over refusal naming the cap, bounded
invalid UTF-8, an oversized regular artifact through scan and doctor,
file- and directory-symlinks at flat positions, a symlinked managed
directory, symlinked containment directories, `sprint.md`, and task
files (scan errors and doctor findings agree), harvest refusing
symlinked directories/files, ancestor-loop and symlinked-root
tolerance, harvest skipping oversized files while retaining all
duplicate claimants, the claim 4 external-identity edge now dangling,
and CLI-level strict/doctor/`--json` agreement.

Retained limitations and handoffs: the cap is per-file only — a strict
scan may still retain up to N × cap across N valid artifacts; the
aggregate-retention seam stays deferred per thread 8 on idea 18. The
harvest retains every claimant in deterministic order; the first-wins
collapse survives only inside `harvest_ids` at its single caller
boundary, explicitly marked as task 23's replacement point. Identity
resolution policy is unchanged here.