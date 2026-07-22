---
id: tsk-license-the-project
sequence: 9
kind: task
status: pending
sprint: spr-community-standards
created: 2026-07-22
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
