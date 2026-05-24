+++
id = "4691685e"
title = "support for worker_profile manifest"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4691685e-support-for-worker-profile-manifest"
created_at = "2026-05-24T19:18:32.809526Z"
updated_at = "2026-05-24T19:53:01.011096Z"
+++

## Spec

### Problem

APM currently supports a global `[workers]` config in `.apm/config.toml` and a per-machine `local.toml` override, but there is no way to configure properties per worker profile. All profiles (`claude/worker`, `claude/spec-writer`, etc.) share the same `model`, `env`, and `container` values. This means that if a project wants the spec-writer to use a more capable model (e.g., Opus) while keeping the worker on a faster, cheaper one (e.g., Sonnet), there is no supported way to express that.

The fix is to introduce optional per-profile manifest files at `.apm/agents/<agent>/<role>.toml`. When present, these files supply profile-specific overrides for `model` and `env` that take effect at worker spawn time — in `apm start`, `apm work`, and the server's UI dispatcher — without changing any other behaviour.

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
| 2026-05-24T19:18Z | — | new | philippepascal |
| 2026-05-24T19:34Z | new | groomed | philippepascal |
| 2026-05-24T19:53Z | groomed | in_design | philippepascal |