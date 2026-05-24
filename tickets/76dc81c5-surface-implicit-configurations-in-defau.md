+++
id = "76dc81c5"
title = "surface implicit configurations in default config"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/76dc81c5-surface-implicit-configurations-in-defau"
created_at = "2026-05-24T19:24:15.489361Z"
updated_at = "2026-05-24T19:57:02.661607Z"
+++

## Spec

### Problem

When `apm init` runs, it writes `.apm/config.toml` with only the parameters needed to get started: project identity, ticket and worktree paths, and a few agent/worker knobs. Eight additional configuration sections — `[sync]`, `[git_host]`, `[server]`, `[context]`, `[isolation]`, `[work]` — and several optional fields within the sections that _are_ shown (`[agents].side_tickets`, `[agents].skip_permissions`, `[workers].container`, `[workers].env`, `[workers].keychain`) are written without any mention, with their defaults in effect but invisible.

A new user inspecting the freshly-written config has no way to discover these knobs without reading the Rust source or searching documentation. The fix is to include every implicit parameter in the generated file as commented-out TOML, each annotated with its default value and a one-line description — the pattern used by many well-known tools (Cargo, Redis, Postgres). The file stays functional as-is; the comments are a self-contained reference.

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
| 2026-05-24T19:24Z | — | new | philippepascal |
| 2026-05-24T19:34Z | new | groomed | philippepascal |
| 2026-05-24T19:57Z | groomed | in_design | philippepascal |