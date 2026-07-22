---
id: tsk_01KY61X8H0QY2FZN4JSNP519A8
sequence: 22
kind: task
status: pending
sprint: spr_01KY61D615FAC8VVSTD7QXX1DW
created: 2026-07-22
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

## Result
