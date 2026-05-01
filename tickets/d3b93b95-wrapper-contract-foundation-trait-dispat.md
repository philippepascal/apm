+++
id = "d3b93b95"
title = "Wrapper contract foundation: trait, dispatcher, claude built-in (refactor)"
state = "specd"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/d3b93b95-wrapper-contract-foundation-trait-dispat"
created_at = "2026-04-30T20:01:55.080870Z"
updated_at = "2026-05-01T01:16:52.089903Z"
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
- [ ] All eleven contract env vars are present on the spawned child process: `APM_AGENT_NAME`, `APM_TICKET_ID`, `APM_TICKET_BRANCH`, `APM_TICKET_WORKTREE`, `APM_SYSTEM_PROMPT_FILE`, `APM_USER_MESSAGE_FILE`, `APM_SKIP_PERMISSIONS` (`"1"` or `"0"`), `APM_PROFILE`, `APM_WRAPPER_VERSION=1`, `APM_BIN` (canonicalized path of the running APM binary); `APM_ROLE_PREFIX` is set when `ctx.role_prefix.is_some()`
- [ ] System prompt content is written to a temp file before spawn; `ctx.system_prompt_file` and `APM_SYSTEM_PROMPT_FILE` point to the same path
- [ ] User message content is written to a temp file before spawn; `ctx.user_message_file` and `APM_USER_MESSAGE_FILE` point to the same path
- [ ] Both temp files are removed after the child process exits (best-effort; removal errors are not propagated)
- [ ] `build_spawn_command` is refactored to write temp files and dispatch through `WrapperContext`; it no longer directly appends `--output-format`, `--verbose`, `--system-prompt`, or `--dangerously-skip-permissions` to the command
- [ ] `spawn_container_worker` is refactored to write temp files and dispatch through `WrapperContext`; docker `--env` flags carry the same APM contract vars as the local path
- [ ] All pre-existing tests in `start.rs` pass
- [ ] New unit tests cover: `resolve_builtin` returning `Some`/`None`, all APM env vars present on the spawned process (including `APM_BIN`), temp file creation and best-effort cleanup after child exit

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

**New module: `apm-core/src/wrapper/`**

Create two files; register with `pub mod wrapper;` in `apm-core/src/lib.rs`:

- `wrapper/mod.rs` — `Wrapper` trait, `WrapperContext`, `resolve_builtin()`
- `wrapper/claude.rs` — `ClaudeWrapper` struct implementing `Wrapper`

---

**`WrapperContext` (in `wrapper/mod.rs`)**

Public struct with fields: `worker_name: String`, `ticket_id: String`, `ticket_branch: String`, `worktree_path: PathBuf`, `system_prompt_file: PathBuf`, `user_message_file: PathBuf`, `skip_permissions: bool`, `profile: String`, `role_prefix: Option<String>`, `options: HashMap<String, String>` (empty until config ticket 6cac8518), `model: Option<String>`, `log_path: PathBuf`, `container: Option<String>` (Some(image) → docker path).

**`Wrapper` trait**

One method: `fn spawn(&self, ctx: &WrapperContext) -> anyhow::Result<std::process::Child>`

**`resolve_builtin`**

Match on name: `"claude"` → `Some(Box::new(ClaudeWrapper))`, anything else → `None`.

---

**`ClaudeWrapper` (in `wrapper/claude.rs`)**

`spawn()` branches on `ctx.container`:

_Local path (container is None):_
1. Read file contents: `sys = fs::read_to_string(&ctx.system_prompt_file)?`, `msg = fs::read_to_string(&ctx.user_message_file)?`
2. Build `Command::new("claude")` with args in order: `--print`, `--output-format stream-json`, `--verbose`, `--system-prompt <sys>`, optionally `--model <value>`, optionally `--dangerously-skip-permissions` (when `ctx.skip_permissions`), then `<msg>` as positional arg
3. Set all APM contract env vars (see table below)
4. `.current_dir(&ctx.worktree_path)`
5. Redirect stdout + stderr to `File::create(&ctx.log_path)?`; `try_clone()` for stderr fd
6. `.process_group(0)`, then `.spawn()`

_Container path (container is Some(image)):_
Build `docker run --rm --volume <wt>:/workspace --workdir /workspace` followed by `--env KEY=VAL` for each APM contract var and inherited vars (ANTHROPIC_API_KEY, git identity), then `<image> claude --print --output-format stream-json --verbose --system-prompt <sys> [--model X] [--dangerously-skip-permissions] <msg>`. Mirrors current `spawn_container_worker` structure; ANTHROPIC_API_KEY and git identity vars still resolved the same way.

**APM contract env vars (set in both paths)**

Compute `apm_bin` once before spawning: `let apm_bin = std::env::current_exe()?.canonicalize()?.to_string_lossy().into_owned();`

| Var | Value |
|---|---|
| `APM_AGENT_NAME` | `ctx.worker_name` |
| `APM_TICKET_ID` | `ctx.ticket_id` |
| `APM_TICKET_BRANCH` | `ctx.ticket_branch` |
| `APM_TICKET_WORKTREE` | `ctx.worktree_path` as str |
| `APM_SYSTEM_PROMPT_FILE` | `ctx.system_prompt_file` as str |
| `APM_USER_MESSAGE_FILE` | `ctx.user_message_file` as str |
| `APM_SKIP_PERMISSIONS` | `"1"` or `"0"` |
| `APM_PROFILE` | `ctx.profile` |
| `APM_ROLE_PREFIX` | `ctx.role_prefix` when `ctx.role_prefix.is_some()` |
| `APM_WRAPPER_VERSION` | `"1"` |
| `APM_BIN` | canonicalized path of the running APM binary (`current_exe().canonicalize()`) |

For local path, use `.env(key, val)` on `Command`. For container path, use `--env key=val` docker args.

---

**Temp file helpers (private, in `start.rs` or `wrapper/mod.rs`)**

`write_temp_file(prefix: &str, content: &str) -> Result<PathBuf>`: write content to `std::env::temp_dir() / "apm-{prefix}-{random}.txt"` and return the path. Use `rand_u16()` (already exists) for the suffix.

---

**Refactoring `build_spawn_command` and `spawn_container_worker`**

Replace both with a single private `spawn_worker(ctx: WrapperContext) -> Result<Child>` that calls `resolve_builtin("claude").expect("always registered").spawn(&ctx)`. The three call sites (`run()`, `run_next()`, `spawn_next_worker()`) each gain the same pattern:

1. Write temp files: `sys_file = write_temp_file("sys", &worker_system)?`, `msg_file = write_temp_file("msg", &ticket_content)?`
2. Build `WrapperContext` from locals already in scope — all fields (`ticket_id`, `ticket_branch`, `worktree_path`, `profile` name, `role_prefix`, `model`, `container`, etc.) are available at each call site
3. Call `spawn_worker(ctx)` → `child`
4. Spawn a cleanup thread that waits on `child.wait()` then calls `fs::remove_file` on both temp paths (errors ignored)

`params.args` (the `--print` arg previously from `workers.args` config) is no longer passed to the claude CLI from outside; the built-in hardcodes it. The `params.env` user-configured env vars should still be forwarded to the child process; add them to `WrapperContext.options` or thread them through a dedicated `extra_env: HashMap<String,String>` field.

---

**Tests to add**

- `resolve_builtin_claude_returns_some` — `assert!(resolve_builtin("claude").is_some())`
- `resolve_builtin_unknown_returns_none` — `assert!(resolve_builtin("bogus").is_none())`
- `claude_wrapper_sets_apm_env_vars` — mock script writes its env to a file; assert `APM_TICKET_ID`, `APM_TICKET_BRANCH`, `APM_TICKET_WORKTREE`, `APM_SYSTEM_PROMPT_FILE`, `APM_USER_MESSAGE_FILE`, `APM_SKIP_PERMISSIONS`, `APM_PROFILE`, `APM_WRAPPER_VERSION`, `APM_BIN` are all present with correct values (same fixture pattern as existing `spawn_worker_cwd_is_ticket_worktree`); also assert `APM_BIN` points to an existing file
- `temp_files_removed_after_child_exits` — write two temp files, include their paths in `WrapperContext`, spawn a trivial wrapper, wait, assert both files are gone

**Existing test compatibility:** `spawn_worker_cwd_is_ticket_worktree` calls `build_spawn_command` directly. After the refactor, update it to call `spawn_worker` with a `WrapperContext`; the invariant (cwd == worktree path) is unchanged.

### Open questions


### Amendment requests

- [x] Add `APM_BIN=<absolute path>` to the wrapper-contract env var table. Source the value from `std::env::current_exe()` (same call that stamps `APM_WRAPPER_VERSION`) and resolve symlinks via `canonicalize`. This gives wrappers (especially the mocks in 25c92daa, but also any future custom wrapper that wants to call back into apm) a deterministic path to the same binary that spawned them. Without this, wrappers either rely on PATH (broken in tests, fragile across multi-version installs) or have to be told the path some other way per-wrapper. Set it in both spawn paths (container and non-container) and document it in the env-var table in the Approach.
- [x] Tighten the `APM_ROLE_PREFIX` AC to match the Approach: it is set when `ctx.role_prefix.is_some()`. The current AC says "when configured" without naming the condition; make it explicit so the implementer doesn't have to interpret.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-30T20:01Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-04-30T21:02Z | groomed | in_design | philippepascal |
| 2026-04-30T21:08Z | in_design | specd | claude-0430-2102-93c0 |
| 2026-05-01T01:10Z | specd | ammend | philippepascal |
| 2026-05-01T01:14Z | ammend | in_design | philippepascal |
| 2026-05-01T01:16Z | in_design | specd | claude-0501-0114-d728 |
