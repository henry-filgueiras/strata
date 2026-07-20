---
id: tsk-bootstrap-create-dragon
sequence: 3
kind: task
status: pending
sprint: spr-bootstrap
created: 2026-07-20
---

# Create a numbered dragon artifact

## Objective

Implement:

```sh
strata new dragon "Branch sequence collisions"
```

## Acceptance criteria

- Locates the repository root.
- Scans existing dragon display sequences.
- Allocates the next sequence safely.
- Generates a deterministic lowercase kebab-case filename.
- Assigns a stable identity.
- Writes valid Markdown front matter and template sections.
- Never overwrites an existing artifact.
- Failed creation does not leave a partial destination file.
