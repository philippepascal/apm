+++
id = "92b34957"
title = "claude seem to think sync is needed to list tickets."
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/92b34957-claude-seem-to-think-sync-is-needed-to-l"
created_at = "2026-06-09T21:59:01.000886Z"
updated_at = "2026-06-12T08:00:03.937302Z"
+++

## Spec

### Problem

Agent instructions in three shipped role files teach Claude that `apm sync` must run before `apm list`. The Shell Discipline section of `apm.main-agent.md`, `apm.spec-writer.md`, and `apm.coder.md` all use `apm sync && apm list --state ready` as the canonical "wrong chaining" example, with the "right" version showing the two commands as sequential calls. This trains every Claude role — not just the main agent — to treat sync as a list prerequisite.

The main-agent startup sequence reinforces the false dependency by placing `apm sync` directly before `apm list --state in_progress` and describing it as "refresh local cache from all `ticket/*` branches". There is no filesystem cache; `apm list` reads git refs directly and returns results whether or not sync has been run. The description misleads agents into thinking list depends on a cache that sync populates.

A third, weaker signal: `apm list` prints "local ref behind origin — run `apm sync` to fast-forward" when stale refs are detected. Alone this would read as a suggestion, but combined with the instruction patterns above it reads as confirmation of the supposed dependency.

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
| 2026-06-09T21:59Z | — | new | philippepascal |
| 2026-06-12T07:52Z | new | groomed | philippepascal |
| 2026-06-12T08:00Z | groomed | in_design | philippepascal |