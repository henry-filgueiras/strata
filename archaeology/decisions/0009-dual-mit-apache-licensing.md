---
id: dec-dual-mit-apache-licensing
sequence: 9
kind: decision
status: accepted
created: 2026-07-22
---

# Dual-license under MIT OR Apache-2.0

## Context

The repository had no license, which defaults to all rights reserved —
in spirit the opposite of the not-hostage invariant (decision 1's
premise that a Strata repository must remain usable without Strata's
blessing extends naturally to Strata itself remaining usable without
the author's). GitHub's community-standards checklist made the gap
visible; the real deadline is that relicensing after external
contributions requires every contributor's consent, so the choice is
cheapest exactly now, while there are none.

Candidates considered:

- **MIT alone** — maximal simplicity and compatibility, no patent
  grant.
- **Apache-2.0 alone** — explicit patent grant and contribution terms,
  but GPLv2-incompatible, which the MIT arm of a dual license repairs.
- **MIT OR Apache-2.0** — the Rust ecosystem convention: rustc, the
  standard library, and every dependency currently in `Cargo.toml`
  (clap, serde, jiff, thiserror, toml, ulid, tempfile) use it.
  Downstream users pick whichever arm suits them.
- **Copyleft (GPL/MPL)** — rejected; a tool whose pitch is "your
  repository stays yours, in ordinary files" should not attach viral
  terms, and ecosystem friction would be high for a library-shaped
  future (decision 7 anticipates Strata-as-substrate).

## Decision

License the project MIT OR Apache-2.0, expressed the standard Rust way:

- `LICENSE-MIT` and `LICENSE-APACHE` at the repository root, unmodified
  standard texts;
- `license = "MIT OR Apache-2.0"` in `Cargo.toml`;
- a README License section carrying the conventional inbound clause:
  contributions intentionally submitted for inclusion are dual-licensed
  the same way unless explicitly stated otherwise (the Apache-2.0
  §5 default, made visible).

## Consequences

- crates.io publication is unblocked from a licensing standpoint
  (publication itself remains a non-goal of sprint 3).
- Inbound contributions arrive under known terms; no CLA machinery is
  needed at this scale.
- Changing the license later requires consent from all contributors to
  date — this decision should be treated as effectively permanent once
  the first external contribution lands.
- The dual-file layout means GitHub reports both licenses rather than
  one SPDX id; this is the ecosystem-standard presentation and how
  serde, clap, and rustc appear.
