---
id: spr-community-standards
sequence: 3
kind: sprint
status: active
created: 2026-07-22
---

# Sprint 3: Community standards

## Goal

Satisfy GitHub's community-standards checklist for the repository:
license, code of conduct, contributing guide, security policy, issue and
pull request templates, plus the two settings-only items (repository
description, content-report acceptance) that have no file representation.

## Rationale

The repository is public and self-describing but legally and socially
unfinished: no license (default "all rights reserved" contradicts the
not-hostage invariant in spirit), no stated conduct or contribution
terms, and no structured entry point for outside reports. GitHub's
community profile checklist is the external forcing function; the work
is worth doing on its own merits before any outside contributor or
crates.io publication arrives, because relicensing after external
contributions requires contributor consent.

This sprint deliberately runs concurrently with sprint 2
(`spr-lifecycle-and-recall`), which remains the engineering sprint. The
framework to date assumes a singular "current sprint" (CLAUDE.md's
workflow says "read the current sprint"); this is the first test of two
active sprints with disjoint scope. Observation for the eventual sprint
model: concurrency was unproblematic to *declare* — the open question is
whether tooling (`doctor`, a future `strata sprint` command) should
treat multiple active sprints as valid, advisory, or diagnosable.

A second taxonomy observation: two checklist items are GitHub settings,
not repository files. The framework has no executor concept — nothing
distinguishes a task an agent can complete from one only a human with
web-UI access can. Task 12 models this as a task that stays pending
until its human performs it; whether tasks deserve an executor or
capability field is left as an observation, not a proposal.

## Success criteria

- The repository is dual-licensed MIT OR Apache-2.0 per decision 9,
  with `LICENSE-MIT` and `LICENSE-APACHE` at the root, the `license`
  field in `Cargo.toml`, and a License section in the README including
  the standard inbound-contribution clause.
- `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`, and `SECURITY.md` exist at
  the repository root and are accurate about this project's actual
  workflow (archaeology updates, `scripts/check.sh`, commit style).
- `.github/ISSUE_TEMPLATE/` provides bug-report and idea templates that
  mirror the archaeology taxonomy rather than generic boilerplate;
  `.github/PULL_REQUEST_TEMPLATE.md` exists.
- GitHub's community-standards checklist shows every file-backed item
  green once pushed.
- The settings-only items (description, content-report toggle) are
  recorded as a pending human task, not silently dropped.

## Non-goals

- Publishing to crates.io (licensing merely unblocks it).
- CI badges, coverage, or release automation.
- Governance documents beyond the checklist (maintainer ladders,
  RFC processes) — single-maintainer project.
- Any Strata code changes; sprint 2 owns the engineering surface.
- Automation that validates community files (`doctor` stays scoped to
  the archaeology).

## Amendments

- 2026-07-22: both taxonomy observations in the rationale were promoted
  to parked ideas after owner review, so "an observation, not a
  proposal" no longer describes them. Sprint-concurrency semantics —
  concurrency valid, cross-sprint hard dependencies into unsettled work
  a deterministic failure, an empty opening frontier advisory — are
  idea 14 (`idea-cross-sprint-dependency-validity`). The executor gap
  became a capability-affordance model with an up-for-grabs filter,
  idea 15 (`idea-capability-constrained-work`). This sprint's own
  concurrency is valid under idea 14's proposed rule: its scope is
  disjoint from sprint 2's and no cross-sprint dependency exists.
