---
id: tsk_01KY7S6QCN94NTTZ8CEW8RQS77
sequence: 35
kind: task
status: pending
sprint: spr_01KY7S6Q69YJ6HATZB48SZBRRM
created: 2026-07-23
---

# Shell completions subcommand

## Objective

Add a `strata completions <shell>` subcommand that emits shell
completion scripts, lowering the cost of human CLI use — the surface
where desire-path data originates.

## Acceptance criteria

- `strata completions zsh` emits a completion script that loads
  without error in zsh; other shells supported by the completion
  generator may be included where they come free.
- The subcommand appears in `--help` with no placeholder flags.
- A brief installation note lands in the README.
- `scripts/check.sh` passes.
