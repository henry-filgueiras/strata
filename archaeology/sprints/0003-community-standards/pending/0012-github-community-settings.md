---
id: tsk-github-community-settings
sequence: 12
kind: task
status: pending
sprint: spr-community-standards
created: 2026-07-22
---

# Flip the settings-only community checklist items

## Objective

Complete the two community-checklist items that exist only as GitHub
settings, with no file representation an agent can write: the
repository description and the "repository admins accept content
reports" toggle.

This task is human-performed by design — it requires web-UI access to
repository settings. It exists so the settings-only items are tracked
rather than silently dropped, and as the framework's first specimen of
work an agent cannot execute (see the sprint rationale's executor-gap
observation).

## Acceptance criteria

- The GitHub repository description is set (suggested: "Git-friendly
  project archaeology and structured repository memory for humans and
  coding agents").
- "Repository admins accept content reports" is enabled in the
  repository's moderation settings (Settings → Moderation options →
  Reported content).
- The community-standards checklist shows both items green.
- The enforcement contact in `CODE_OF_CONDUCT.md` is confirmed or
  changed by the repository owner (an agent chose the default).
