---
id: tsk-contribution-templates
sequence: 11
kind: task
status: closed
sprint: spr-community-standards
created: 2026-07-22
closed: 2026-07-22
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

## Result

All three templates added. The idea template is the interesting one: it
reproduces the idea artifact's Problem / Sketch / Evidence shape and
states the never-load-bearing rule, so triage of an accepted proposal is
"park it into `archaeology/ideas/` nearly verbatim" — the issue tracker
becomes an intake funnel for the taxonomy rather than a parallel
backlog. The bug template asks for `strata doctor` output and flags
content loss as the most serious class, mirroring decision 8's failure
contract. Markdown templates were chosen over GitHub's YAML-form flavor:
they satisfy the checklist, and free-text sections suit reports that
quote front matter and console output.

## Verification

Front matter on both issue templates carries `name`/`about`/`labels`;
paths match GitHub's detection rules (`.github/ISSUE_TEMPLATE/*.md`,
`.github/PULL_REQUEST_TEMPLATE.md`). `scripts/check.sh` clean (docs
only). Checklist detection itself is only observable after push —
recorded as residual verification in task 12.
