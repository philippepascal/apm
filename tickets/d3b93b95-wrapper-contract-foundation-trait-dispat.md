+++
id = "d3b93b95"
title = "Wrapper contract foundation: trait, dispatcher, claude built-in (refactor)"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/d3b93b95-wrapper-contract-foundation-trait-dispat"
created_at = "2026-04-30T20:01:55.080870Z"
updated_at = "2026-04-30T21:02:14.844968Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
+++

## Spec

### Problem

Refactor `apm-core/src/start.rs` to dispatch through a Wrapper abstraction instead of hardcoding Claude flags. After this ticket, behaviour is byte-for-byte identical to today (same flags, same input/output) but the code path is wrapper-driven and the foundation is in place for additional built-ins and project-defined wrappers.

**Reference spec:** `docs/agent-wrappers.md` — sections 'Why', 'Overall design', 'The wrapper contract'.

**Scope:**
- New module `apm-core/src/wrapper/` (or similar). Define a `Wrapper` trait with one method (e.g. `spawn(&self, ctx: WrapperContext) -> Result<Child>`).
- `WrapperContext` carries: ticket id, ticket branch, worktree path, system-prompt file path, user-message file path, skip_permissions flag, profile name, role_prefix, options map, model.
- A `BuiltinRegistry` enum/map; the only entry registered in this ticket is `claude`.
- The `claude` built-in invokes `claude --print --output-format=stream-json --verbose --system-prompt <file-contents> [--model X] [--dangerously-skip-permissions] <user-message>` exactly as today, producing identical output behaviour.
- Refactor `spawn_container_worker()` and `build_spawn_command()` in `start.rs` to: (a) write system prompt + user message to temp files, (b) set the env-var contract from the spec ('The wrapper contract' section), (c) dispatch to the registered wrapper. The two functions become thin glue.
- Env vars to set per spec: `APM_AGENT_NAME`, `APM_TICKET_ID`, `APM_TICKET_BRANCH`, `APM_TICKET_WORKTREE`, `APM_SYSTEM_PROMPT_FILE`, `APM_USER_MESSAGE_FILE`, `APM_SKIP_PERMISSIONS`, `APM_PROFILE`, `APM_ROLE_PREFIX`, `APM_WRAPPER_VERSION=1`. `APM_OPT_<KEY>` deferred to ticket-3 (config) since options aren't readable yet.
- chdir to ticket worktree before exec (already happens; preserve).
- Capture stdout + stderr to `.apm-worker.log` (already happens; preserve).
- Remove the temp prompt and message files when the worker exits (best-effort).

**Out of scope:**
- Reading wrapper choice from config (ticket comes next; for now, hardcode `claude` as the only wrapper).
- Custom wrappers from `.apm/agents/<name>/` (separate ticket).
- Mock built-ins (separate ticket).
- Wrapper-contract versioning checks (separate ticket; just stamp `APM_WRAPPER_VERSION=1`).
- The `check_output_format_supported()` removal — keep it for now; it can be removed when config moves to `agent` field.

**Tests:** existing worker-spawn tests must still pass byte-for-byte. Add unit tests for the new `WrapperContext` plumbing.

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
| 2026-04-30T20:01Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
