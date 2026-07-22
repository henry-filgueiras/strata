---
id: ide_01KY5X7C56KBFWJJJKHTEXXQXV
sequence: 18
kind: idea
status: parked
created: 2026-07-22
---

# Modification watermark for scan performance

## Problem

If collections abandon lifecycle subdirectories for flat per-collection
directories (sprint 5's placement question), every status-filtered
query — "open dragons", "parked ideas", the most common operations —
must parse front matter for the whole collection, including an
ever-growing long tail of terminal-state artifacts. At today's scale
this is microseconds; a repository with thousands of closed artifacts
pays a linear scan for answers that concern only the active few.

## Sketch

A progressively updated watermark: a small generated file per
collection recording the modification timestamp of the oldest
non-terminal artifact. A scanner may skip any file older than the
watermark when it only needs non-terminal artifacts, because everything
past it is known-terminal. The watermark is a disposable projection in
the decision 1 sense — deletable at any time, rebuilt by one full scan,
never canonical, and `doctor` can verify it cheaply. Do not build it
until a real repository demonstrates a felt scan cost; the flat-layout
decision should only note the seam exists.

## Evidence

Raised by Henry while weighing flat placement for sprint 5: the known
cost of one flat directory per collection is that status filters read
everything, and this is the counter-lever if it ever hurts. Prior art:
mail dir readers and build systems skipping by mtime watermark.
[[idea-declarative-collection-specs|Declarative collection specs]]
would give the watermark one generic implementation point instead of
per-collection ones.
