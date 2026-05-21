+++
id = "fcf0b213"
title = "apm command to feed apm parsed help for agents"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/fcf0b213-apm-command-to-feed-apm-parsed-help-for-"
created_at = "2026-05-07T20:41:08.889701Z"
updated_at = "2026-05-21T23:00:22.336234Z"
+++

## Spec

### Problem

Agent instruction files (e.g., `.apm/agents/claude/apm.worker.md`) currently contain manually maintained summaries of available APM commands. These summaries are vague and drift out of sync as commands are added, renamed, or gain new flags. As a result, agents operating from stale instructions may invoke wrong syntax, miss new flags, or apply workarounds for limitations that no longer exist.

The accurate, complete command metadata already exists in the clap command definitions — the same source that powers `apm help commands`. A new `apm instructions` command exposes that metadata as a compact, plain-text guide. Agents can call it at startup or on demand to get an authoritative, always-current reference without relying on hardcoded prose.

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
| 2026-05-07T20:41Z | — | new | philippepascal |
| 2026-05-21T22:59Z | new | groomed | philippepascal |
| 2026-05-21T23:00Z | groomed | in_design | philippepascal |