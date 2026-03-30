+++
id = "37c280f2"
title = "apm take should work without APM_AGENT_NAME set"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "20089"
branch = "ticket/37c280f2-apm-take-should-work-without-apm-agent-n"
created_at = "2026-03-30T14:41:58.874046Z"
updated_at = "2026-03-30T16:37:04.724336Z"
+++

## Spec

### Problem

Currently, `apm take` hard-fails with `APM_AGENT_NAME is not set` if the environment variable is absent (take.rs lines 7–8). This is inconsistent with other commands: `apm state`, `apm close`, and `apm start` all fall back gracefully via `unwrap_or_else` or the `resolve_agent_name()` helper (APM_AGENT_NAME → $USER → $USERNAME → literal "apm").

`apm take` is typically used by supervisors or engineers reclaiming a stalled ticket. These callers may not have exported `APM_AGENT_NAME`. The hard failure forces an extra export step that has no real safety benefit, since the same agent-name resolution logic already exists in the codebase.

The desired behaviour is that `apm take` uses the same `resolve_agent_name()` helper from `start.rs` so it succeeds whenever any reasonable identity is available, and only produces a generic fallback when no env vars are set at all.

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T14:41Z | — | new | philippepascal |
| 2026-03-30T16:37Z | new | in_design | philippepascal |