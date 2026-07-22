---
id: tsk-license-the-project
sequence: 9
kind: task
status: closed
sprint: spr-community-standards
created: 2026-07-22
closed: 2026-07-22
---

# License the project under MIT OR Apache-2.0

## Objective

Give the repository an explicit outbound license per decision 9
(`dec-dual-mit-apache-licensing`), ending the default all-rights-reserved
state before any external contribution arrives.

## Acceptance criteria

- `LICENSE-MIT` and `LICENSE-APACHE` exist at the repository root with
  the standard, unmodified license texts (MIT with copyright holder and
  year; Apache License 2.0).
- `Cargo.toml` declares `license = "MIT OR Apache-2.0"`.
- The README gains a License section stating the dual license and the
  standard clause that inbound contributions are dual-licensed the same
  way unless explicitly stated otherwise.
- `scripts/check.sh` passes (the `Cargo.toml` edit must not disturb the
  build).

## Result

Dual-licensed per decision 9 (`dec-dual-mit-apache-licensing`).
`LICENSE-MIT` carries the MIT text with "Copyright (c) 2026 Henry
Filgueiras"; `LICENSE-APACHE` is the canonical text downloaded from
apache.org rather than reproduced by hand — its sha256
(`cfc7749b96f6…`) matches the well-known canonical hash. `Cargo.toml`
gained `license`, and opportunistically `description` and `repository`,
since the description doubles as the suggested GitHub repository
description in task 12. The README License section uses the
ecosystem-standard wording including the inbound dual-licensing clause.

## Verification

`scripts/check.sh` clean (fmt, tests, clippy). `cargo metadata` parses
the new package fields. License texts verified against canonical
sources, not regenerated.
