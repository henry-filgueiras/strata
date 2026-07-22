---
id: idea-strata-fortune
sequence: 6
kind: idea
status: adopted
created: 2026-07-20
adopted: 2026-07-22
adopted-by: "[[tsk-strata-fortune|ambient recall task]]"
---

# `strata fortune`: ambient recall of open risks

## Problem

Paper trails fail on the read side. A risk gets recorded precisely so it
will not be forgotten — and then it is forgotten, because nothing ever
resurfaces it. Open dragons are exactly the artifacts whose value depends
on periodic re-encounter, and today the only read paths are deliberate
(`list`, `show`), which require already remembering that there is
something to remember.

## Sketch

`strata fortune` prints one artifact excerpt — title, human reference,
age, and a snippet — drawn from open dragons and parked ideas. Selection
favors staleness: the least-recently-touched artifact is the one most in
need of daylight (a uniformly random mode keeps it fortune-cookie fun).
Read-only, zero new state, a pure projection.

The adoption hook is the placement, not the command: one line in a shell
greeting, an MOTD, a CI job summary, or an agent's session preamble turns
the archive into ambient memory. "Your repo remembers a dragon you don't"
is the cheapest possible demo of the whole thesis — and for coding
agents, a fortune at session start is a zero-cost recall channel for
risks that would otherwise wait for someone to think of grepping.

Deliberately silly surface, deliberately serious mechanism.

## Evidence

Dragons exist to keep unresolved risks visible (CLAUDE.md, project
purpose); derived-projections invariant makes the command nearly free
(decision 1, `dec-bootstrap-files-canonical`); the read-rate concern —
records nobody re-reads are a diary, not memory — motivated the
case-study framing of this repository. Prior art: `fortune(6)`, MOTD,
Oblique Strategies, spaced repetition (staleness-weighted selection is
the same instinct pointed at risk registers).

## Adoption (2026-07-22)

Adopted by task 8 (`tsk-strata-fortune`), which landed `strata fortune`
in sprint 2. Divergences from the sketch:

- v1 draws only from open dragons; parked ideas join the pool when
  ideas become a managed collection (per the sprint 2 non-goal).
- Selection is staleness-weighted with every open dragon reachable —
  the sketch's "least-recently-touched" extreme and the uniform
  fortune-cookie mode both collapsed into one weighted draw; age comes
  from `created`, since no touch metadata exists.
- Per the task 8 amendment (thread `cmt-fortune-reproducibility`),
  `--seed`, `--json`, and any selection-metadata surface are
  deliberately excluded until a real automation consumer exists.

The adoption hooks (shell greeting, MOTD, session preambles) remain
placement suggestions, not shipped configuration.
