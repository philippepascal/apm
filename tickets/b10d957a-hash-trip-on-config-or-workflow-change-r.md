+++
id = "b10d957a"
title = "Hash-trip on config or workflow change runs apm validate"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b10d957a-hash-trip-on-config-or-workflow-change-r"
created_at = "2026-04-27T20:28:59.343081Z"
updated_at = "2026-04-27T20:28:59.343081Z"
epic = "5ea30227"
target_branch = "epic/5ea30227-strategy-and-dependency-hardening"
depends_on = ["e845127e"]
+++

## Spec

### Problem

Strategy or workflow changes can silently invalidate existing dependency setups (e.g., a ticket with deps was created under `merge`, then the strategy is switched to `pr` where deps are not allowed). The spec at `docs/strategy-and-dependencies.md` (section 'Hash-trip on config change') requires APM to detect config drift and refresh validation automatically.

Hash `.apm/config.toml` and `.apm/workflow.toml` and store the stamp in `.apm/.validate-stamp` (gitignored). On every `apm` invocation, compare the live hash to the stamp:
- If unchanged: no-op (cost: stat + hash, microseconds)
- If changed: run `apm validate` automatically. If validation fails, refuse mutating commands (`apm new`, `apm state`, `apm set`, `apm spec`, `apm start`) until the issue is resolved; warn but allow read-only commands (`apm list`, `apm show`, `apm next`). On pass, refresh the stamp.

Hash function should be cheap and stable (e.g., blake3 or sha2 over the file bytes). Depends on ticket e845127e — `apm validate` must enforce the dependency rules first, otherwise the hash-trip has nothing meaningful to surface.

See docs/strategy-and-dependencies.md, section 'Hash-trip on config change'.

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
| 2026-04-27T20:28Z | — | new | philippepascal |
