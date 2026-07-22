---
id: tsk-community-health-files
sequence: 10
kind: task
status: pending
sprint: spr-community-standards
created: 2026-07-22
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
