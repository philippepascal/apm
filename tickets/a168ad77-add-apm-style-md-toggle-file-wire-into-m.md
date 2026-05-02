+++
id = "a168ad77"
title = "Add .apm/style.md toggle file; wire into main agent and spec-writer"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a168ad77-add-apm-style-md-toggle-file-wire-into-m"
created_at = "2026-05-01T19:39:37.765619Z"
updated_at = "2026-05-02T03:46:35.921856Z"
+++

## Spec

### Problem

APM agents currently have no mechanism for experimenting with output brevity. Conversation replies can be verbose (preambles, multi-sentence answers to one-liners, end-of-turn check-ins), and spec output can be padded (long Problem sections, AC lists that exceed what's useful). The supervisor wants to compress responses without committing to a full rewrite — trying rules one at a time and keeping only what helps.

A checkbox-toggle file at `.apm/style.md` provides that mechanism. All rules start unchecked; the supervisor checks individual boxes to activate them and unchecks them if they cause problems. The file is read by the main agent on every session (via a `CLAUDE.md` import) and by the spec-writer before writing or amending a spec (via the per-agent instruction files introduced by the wrapper epic, ticket 4312fbd4). No code parses the file; it is read directly by the agents as part of their prompt context.

**Dependency:** this ticket must land after the wrapper epic (4312fbd4), which creates the `.apm/agents/claude/` per-agent layout that the spec-writer `.md` change targets.

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
| 2026-05-01T19:39Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:46Z | groomed | in_design | philippepascal |