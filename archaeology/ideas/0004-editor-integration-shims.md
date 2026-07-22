---
id: idea-editor-integration-shims
sequence: 4
kind: idea
status: parked
created: 2026-07-20
---

# Editor integrations as thin shims over the CLI

## Problem

Reference completion and write-time binding need candidate data inside the
editor, but decision 0007 forbids resolution logic living in editor code:
human and machine callers must share one core.

## Sketch

Editor packages (Emacs first) implement completion-at-point for artifact
references by shelling out to the CLI — `strata list --json` as the
candidate feed, the eventual bind operation for insertion — inserting the
canonical ID-plus-frozen-label form at completion accept. No parsing, no
resolution, no state in the editor; a different editor is a different
twenty-line shim.

## Evidence

Decision 0007 (`dec-bootstrap-interaction-surfaces`); decision 0006
(`dec-bootstrap-reference-model`, write-time binding at authorship);
prior art in org-roam's ID-targeted completion.
