---
id: idea-launch-channels
sequence: 16
kind: idea
status: parked
created: 2026-07-22
---

# Launch channels: publicizing Strata once the loop demos itself

## Problem

The tool has no distribution soapbox: discoverability today is entirely
organic search over the repository. The owner's channel constraints are
real and worth recording so a future launch doesn't rediscover them: no
Hacker News account (rules out Show HN directly), and a long-lived but
participation-less Reddit account, which some subreddits' karma or
activity gates have blocked before — r/rust's current posting policy
needs checking before counting on it. The readiness bar, set by the
owner: wait until Strata has a good enough exemplified loop *through
the tool itself* — the repository demonstrating fortune, transitions,
and doctor on its own archaeology is the product demo.

## Sketch

A staged inventory of cheap, high-fit channels, in rough order:

1. **GitHub topics** — done (task 12, 2026-07-22): twelve topics, with
   `architecture-decision-records`/`adr`/`decision-records` as the
   load-bearing niche where searchers are precisely the audience, and
   `ai-agents`/`agent-memory` as the second audience.
2. **crates.io publish** — unblocked by decision 9's licensing;
   `cargo install strata` is itself a distribution channel and the
   crates.io page ranks well in search. Requires settling the crate
   name's availability first.
3. **README demo asset** — a short GIF or asciinema of the real loop
   (`fortune` surfacing a dragon, `close`, `doctor`) as the anchor
   every other channel links to.
4. **List placements via PR** — the ADR tooling list (adr.github.io)
   and relevant awesome lists (ADR, Rust CLI, agent tooling); these
   are pull-requests, gated on maintainer review rather than accounts
   or karma.
5. **This Week in Rust** — crate-of-the-week nominations are open to
   anyone; no reputation gate.
6. **Reddit/HN** — highest reach, but gated by the account constraints
   above; verify r/rust policy, or a Show HN would need an account
   seasoned in advance. Deliberately last, not first.

Never load-bearing: this is a checklist of options, not a commitment;
adopting it means executing the inventory, and rejecting it costs
nothing canonical.

## Evidence

Owner discussion, 2026-07-22 (constraints and readiness bar are direct
quotes of intent). Task 12 result records the topics as executed.
Decision 9 (`dec-dual-mit-apache-licensing`) explicitly names crates.io
publication as unblocked but out of sprint 3 scope. Prior art: This
Week in Rust's crate of the week as a reputation-free channel;
adr.github.io as the canonical ADR tooling index.
