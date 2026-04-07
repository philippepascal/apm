+++
id = "2bcbd7a4"
title = "create a new authoritative command doc"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/2bcbd7a4-create-a-new-authoritative-command-doc"
created_at = "2026-04-07T17:06:49.569239Z"
updated_at = "2026-04-07T17:43:53.349020Z"
+++

## Spec

### Problem

APM has a rich CLI with ~28 commands, but there is no single authoritative reference document that covers all of them in depth. Existing help text (`--help`) gives one-line descriptions and flag names, but does not explain the internal mechanics—especially the git operations each command performs and why.

Contributors adding new features and users debugging unexpected behaviour have no place to look beyond the source code. A contributor extending `apm sync` needs to understand which `git` calls it already makes and the order they run in; a power user writing a wrapper script needs to know exactly what `apm start` does to a worktree before they can safely automate around it.

The desired outcome is a single Markdown file committed to the repository that serves as the canonical reference for every command: what the command does at a high level, its full argument and flag surface, and a detailed breakdown of every git operation it performs internally with a note on why each one is needed. The format should be inspired by how popular CLI tools like `git` or `curl` document themselves—structured, scannable, and complete enough that reading it once is sufficient to understand the full behaviour.

### Acceptance criteria

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T17:06Z | — | new | philippepascal |
| 2026-04-07T17:42Z | new | groomed | apm |
| 2026-04-07T17:43Z | groomed | in_design | philippepascal |