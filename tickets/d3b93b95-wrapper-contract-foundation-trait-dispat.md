+++
id = "d3b93b95"
title = "Wrapper contract foundation: trait, dispatcher, claude built-in (refactor)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/d3b93b95-wrapper-contract-foundation-trait-dispat"
created_at = "2026-04-30T20:01:55.080870Z"
updated_at = "2026-04-30T21:02:55.193498Z"
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

- [ ] `apm-core/src/wrapper/mod.rs` exists and exports a public `Wrapper` trait with a single method `fn spawn(&self, ctx: &WrapperContext) -> anyhow::Result<std::process::Child>`
- [ ] `WrapperContext` is a public struct with fields covering all items listed in the Problem scope: `worker_name`, `ticket_id`, `ticket_branch`, `worktree_path`, `system_prompt_file`, `user_message_file`, `skip_permissions`, `profile`, `role_prefix`, `options`, `model`, `log_path`
- [ ] `resolve_builtin("claude")` returns `Some(_)` (a `Box<dyn Wrapper>`)
- [ ] `resolve_builtin` returns `None` for any name other than `"claude"`
- [ ] The `claude` built-in spawns `claude --print --output-format=stream-json --verbose --system-prompt <content-of-system-prompt-file> [--model <value>] [--dangerously-skip-permissions] <content-of-user-message-file>` — byte-for-byte identical flags to the current hardcoded invocation
- [ ] All ten contract env vars are present on the spawned child process: `APM_AGENT_NAME`, `APM_TICKET_ID`, `APM_TICKET_BRANCH`, `APM_TICKET_WORKTREE`, `APM_SYSTEM_PROMPT_FILE`, `APM_USER_MESSAGE_FILE`, `APM_SKIP_PERMISSIONS` (`"1"` or `"0"`), `APM_PROFILE`, `APM_WRAPPER_VERSION=1`; `APM_ROLE_PREFIX` is set when `ctx.role_prefix` is `Some`
- [ ] System prompt content is written to a temp file before spawn; `ctx.system_prompt_file` and `APM_SYSTEM_PROMPT_FILE` point to the same path
- [ ] User message content is written to a temp file before spawn; `ctx.user_message_file` and `APM_USER_MESSAGE_FILE` point to the same path
- [ ] Both temp files are removed after the child process exits (best-effort; removal errors are not propagated)
- [ ] `build_spawn_command` is refactored to write temp files and dispatch through `WrapperContext`; it no longer directly appends `--output-format`, `--verbose`, `--system-prompt`, or `--dangerously-skip-permissions` to the command
- [ ] `spawn_container_worker` is refactored to write temp files and dispatch through `WrapperContext`; docker `--env` flags carry the same APM contract vars as the local path
- [ ] All pre-existing tests in `start.rs` pass
- [ ] New unit tests cover: `resolve_builtin` returning `Some`/`None`, all APM env vars present on the spawned process, temp file creation and best-effort cleanup after child exit

### Out of scope

- Reading wrapper name from config (`[workers] agent = ...`) — hardcoded to `claude` for now; config wiring is ticket 6cac8518
- `APM_OPT_<KEY>` env vars — options are not yet readable from config; deferred to ticket 6cac8518
- Custom wrappers from `.apm/agents/<name>/` — ticket 2c32a282
- Mock built-ins (`mock-happy`, `mock-sad`, `mock-random`, `debug`) — ticket 25c92daa
- Wrapper-contract versioning checks against `manifest.toml` — ticket 2e772eab; `APM_WRAPPER_VERSION=1` is stamped but not validated
- Removing `check_output_format_supported()` — kept until config moves to `agent` field
- Per-agent instruction file resolution under `.apm/agents/<name>/` — ticket 7f5f73d5
- `apm agents` subcommand — ticket 71d80e40
- Migration tooling for legacy `command/args/model` config — ticket 3048d7e9
- Frontmatter `agent` / `agent_overrides` fields — ticket 0ca3e019

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
| 2026-04-30T21:02Z | groomed | in_design | philippepascal |