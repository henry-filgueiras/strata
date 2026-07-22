---
id: tsk-contribution-templates
sequence: 11
kind: task
status: pending
sprint: spr-community-standards
created: 2026-07-22
---

# Add issue and pull request templates

## Objective

Give outside reports a structured entry point that mirrors the
archaeology taxonomy instead of generic bug/feature boilerplate, so
triage maps directly onto existing artifact kinds.

## Acceptance criteria

- `.github/ISSUE_TEMPLATE/bug-report.md` asks for the command run,
  repository state, expected and actual behavior, and `strata doctor`
  output where relevant.
- `.github/ISSUE_TEMPLATE/idea.md` mirrors the idea artifact shape
  (Problem / Sketch / Evidence) and states the never-load-bearing rule:
  an accepted idea becomes a parked idea artifact, not a promise.
- `.github/PULL_REQUEST_TEMPLATE.md` includes a short checklist:
  `scripts/check.sh` passes, archaeology updated where the change is
  substantive, commit style followed.
- Templates carry the YAML front matter GitHub requires (`name`,
  `about`) so the checklist detects them.
