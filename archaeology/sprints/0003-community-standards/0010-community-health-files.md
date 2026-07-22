---
id: tsk-community-health-files
sequence: 10
kind: task
status: closed
sprint: spr-community-standards
created: 2026-07-22
closed: 2026-07-22
---

# Add code of conduct, contributing guide, and security policy

## Objective

Add the three narrative community-health files GitHub's checklist looks
for, written for this project rather than pasted boilerplate — the
contributing guide in particular must describe the archaeology workflow,
since that is the part an outside contributor could not guess.

## Acceptance criteria

- `CODE_OF_CONDUCT.md` is the Contributor Covenant v2.1 with a real
  enforcement contact.
- `CONTRIBUTING.md` covers: building and testing (`scripts/check.sh`),
  the archaeology workflow an agent or human is expected to follow
  (read CLAUDE.md and the current sprint; include archaeology updates
  with substantive changes), commit message style, and how to propose
  ideas versus report bugs.
- `SECURITY.md` states the supported-version reality (pre-1.0, latest
  release/main), an honest description of the threat surface (a local
  CLI parsing repository files; no network, no daemon), and a private
  reporting channel (GitHub private vulnerability reporting).
- No file promises process this project does not actually run.

## Result

All three files added at the repository root.

- `CODE_OF_CONDUCT.md` is the Contributor Covenant v2.1 with the
  repository owner's email as enforcement contact — chosen by an agent,
  flagged for confirmation in task 12.
- `CONTRIBUTING.md` leads with what an outsider could not guess: the
  archaeology is load-bearing, substantive changes update it, and the
  same workflow applies to human and agent authors (the case-study
  framing made explicit). It points at CLAUDE.md for invariants rather
  than duplicating them, and at the issue templates for the
  bug-versus-idea split.
- `SECURITY.md` states the honest pre-1.0 posture: no releases, fixes
  on `main`, a deliberately narrow threat surface (untrusted repository
  file parsing, path escape, content loss), private reporting via
  GitHub, acknowledgment within a week, and no promised SLA.

## Verification

`scripts/check.sh` clean (docs only; no code touched). CONTRIBUTING
claims cross-checked against reality: `scripts/check.sh` exists and
enforces fmt/tests/clippy, the cited commit style matches `git log`,
referenced templates land with task 11 in the same sprint.
