+++
id = "6095305a"
title = "Filesystem path validator at wrapper layer (worktree isolation enforcement)"
state = "in_design"
priority = 0
effort = 5
risk = 5
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6095305a-filesystem-path-validator-at-wrapper-lay"
created_at = "2026-05-01T02:30:34.552318Z"
updated_at = "2026-05-02T08:02:47.238266Z"
+++

## Spec

### Problem

Workers spawn inside a dedicated git worktree (`APM_TICKET_WORKTREE`) and are
expected to confine all filesystem writes to that tree. Two gaps make this
expectation unenforceable today:

1. **`-P` workers have no write boundary.** Spawning with
   `--dangerously-skip-permissions` bypasses Claude Code's permission allowlist
   entirely. Such a worker can `Write`, `Edit`, or `Bash`-redirect to any path
   on the filesystem — including the main worktree, other ticket worktrees, or
   paths outside the repo.

2. **Default-permission workers have accidental coverage leaks.** Explicit
   allowlist entries added for legitimate APM paths (e.g. `apm spec` temp
   files, `.apm/` directories) share a prefix with the actual repo root, which
   inadvertently permits writes to the main worktree as well.

The natural enforcement surface is a `PreToolUse` hook: Claude Code invokes a
configured shell command before every tool execution. The command can block the
call by exiting with code 2; Claude Code converts this into a synthetic
`tool_result` error sent back to the model. This hook fires **regardless of
`--dangerously-skip-permissions`**, making it the only mechanism that enforces
write boundaries on both `-P` and non-`-P` workers without changing the process
IPC model or spawning in interactive mode.

This ticket builds the enforcement layer end-to-end: a `PathGuard` struct, an
`apm path-guard` CLI subcommand invoked by the hook, a settings-file writer that
injects the hook before spawning, and an opt-in manifest field for custom
wrappers. No dependency on any other in-flight wrapper ticket is required; the
mechanism is fully self-contained.

### Acceptance criteria

- [ ] `apm start` with `enforce_worktree_isolation = true` spawns the worker with path enforcement active; a worker that issues `Edit` against a path in the main worktree receives a `tool_result` error whose message contains "path outside ticket worktree"
- [ ] The rejection `tool_result` message includes the value of `APM_TICKET_WORKTREE` so the agent can self-correct
- [ ] The main-worktree file targeted by the rejected `Edit` call is unmodified after the rejection
- [ ] A worker issues `Edit` against a path inside `APM_TICKET_WORKTREE`; the call succeeds and the file is modified
- [ ] A worker issues `Write` against a path outside `APM_TICKET_WORKTREE`; the call is rejected with the same error format
- [ ] A worker issues `Bash` with command `echo foo > /path/outside/worktree`; the command is rejected before execution; the target file is unmodified
- [ ] A worker issues `Bash` with command `cat /etc/resolv.conf`; the call is allowed (default read-allow-list entry)
- [ ] A worker issues `Bash` with command `cat ~/.gitconfig`; the call is allowed (default read-allow-list entry)
- [ ] A worker issues `Bash` whose only absolute paths are inside `APM_TICKET_WORKTREE`; the call is allowed
- [ ] A custom wrapper with `enforce_worktree_isolation = false` in its `manifest.toml` runs without path interception; the worker can write outside `APM_TICKET_WORKTREE` unobstructed
- [ ] A custom wrapper whose `manifest.toml` omits `enforce_worktree_isolation` behaves identically to `false` (opt-in, backward-compatible default)
- [ ] Path resolution canonicalises `..` components before comparison; a path like `<worktree>/../../../etc/passwd` is rejected
- [ ] Path resolution follows symlinks before comparison; a symlink inside `APM_TICKET_WORKTREE` that resolves outside it is rejected
- [ ] A `Write` call targeting `APM_BIN` is rejected even when `APM_BIN` has no path relationship to the worktree
- [ ] A `Write` call targeting `APM_SYSTEM_PROMPT_FILE` or `APM_USER_MESSAGE_FILE` is rejected (those paths are read-only exceptions, not writable)
- [ ] The read-allow-list is configurable in `.apm/config.toml` under `[isolation] read_allow`; entries added there permit the corresponding `Bash cat` calls through enforcement

### Out of scope

- Process-level sandboxing (bwrap, sandbox-exec, containers, seccomp) — heavier mechanism; only justified if this tool-level filter proves insufficient
- Network egress filtering — the agent's Anthropic API traffic is out of scope
- Read-only filesystem access outside the worktree — reads are information-only; this ticket blocks writes, not reads (other than through the explicit write-only exceptions)
- Enforcement in custom wrappers that use `parser = "external"` — external parsers implement their own enforcement; the manifest field signals intent but APM core does not enforce on their behalf
- Retroactive enforcement on already-running workers — enforcement applies only to tool calls dispatched after the worker has been spawned with the flag active
- Windows or non-POSIX path handling — all path logic assumes POSIX absolute paths
- Bash false-negative elimination — paths embedded in shell variables, subshell expansions, or indirect redirections will not be caught; documented as a known limitation
- Changing the default of `enforce_worktree_isolation` to `true` — left as a follow-on decision after this ticket ships and any friction is observed

### Approach

**Prerequisite**: The wrapper epic (4312fbd4) must be merged first; it provides
the interception hook that this ticket plugs into. The hook is a callback
registered in `WrapperContext` (or equivalent) that the canonical parser calls
for each `tool_use` event before dispatching execution. The callback receives
the parsed event and returns either `Ok(())` (allow) or `Err(String)` (inject
a synthetic `tool_result` error with that message back to the agent).

---

### 1. `PathGuard` — new module `apm-core/src/wrapper/path_guard.rs`

```rust
pub struct PathGuard {
    worktree: PathBuf,              // canonicalised APM_TICKET_WORKTREE
    read_allow: Vec<PathBuf>,       // paths allowed for read-only Bash cmds
    write_protected: Vec<PathBuf>,  // APM_BIN, APM_SYSTEM_PROMPT_FILE, APM_USER_MESSAGE_FILE
}
```

Key functions:

- `PathGuard::new(ctx: &WrapperContext, cfg: &IsolationConfig) -> Self`
  Canonicalises `ctx.worktree_path` (resolving symlinks). Builds `read_allow`
  from `cfg.read_allow` expanded with `~`. Populates `write_protected` from
  `ctx.system_prompt_file`, `ctx.user_message_file`, and
  `std::env::current_exe()` (APM_BIN).

- `PathGuard::check_write(&self, path: &Path) -> Result<(), String>`
  1. Resolve the path: call `canonicalize_lenient(path)` (see below) to handle
     non-existent targets.
  2. If the resolved path starts with `self.worktree` → `Ok(())`.
  3. If it matches any entry in `self.write_protected` → `Err(rejection_msg)`.
  4. Otherwise → `Err(rejection_msg)`.

- `PathGuard::check_bash(&self, cmd: &str) -> Result<(), String>`
  Extract candidate absolute paths from the command string using the regex
  `/(?:^|[\s=<>|;&`'"])(\~?\/[^\s"';<>|&`]+)/` (find tokens starting with `/`
  or `~/`). For each candidate:
  - Expand `~` to the user's home directory.
  - Check if the token appears to be a write target: present in a redirect
    (`>`, `>>`, `tee`) or in `rm`, `mv`, `cp`, `truncate`, `echo …>`, `cat …>`.
  - If write target: call `check_write(candidate_path)`.
  - If read-only: check against `self.read_allow`; if the resolved path is under
    `self.worktree` or in `read_allow` → allow; else → allow (reads are not
    blocked by this policy).
  Return the first write rejection encountered, or `Ok(())`.

- `canonicalize_lenient(path: &Path) -> PathBuf`
  Walk path components and resolve each symlink that exists; for components that
  don't exist yet (new file writes), resolve `..` lexically without syscalls.
  This avoids TOCTOU between `canonicalize()` failing on absent paths and the
  tool creating them.

**Rejection message template**:
```
path outside ticket worktree; isolation enforced by APM wrapper.
  Requested: {requested_path}
  APM_TICKET_WORKTREE = {worktree}
```

---

### 2. `IsolationConfig` — extend `apm-core/src/config.rs`

Add a new optional table to `ApmConfig`:

```toml
# .apm/config.toml
[isolation]
read_allow = [
  "/etc/resolv.conf",
  "~/.gitconfig",
  "~/.ssh/config",
  "/etc/ssl/certs/**",
]
```

```rust
#[derive(Deserialize, Default)]
pub struct IsolationConfig {
    #[serde(default)]
    pub read_allow: Vec<String>,   // glob patterns; ~ is expanded
}
```

Default (when `[isolation]` is absent): `read_allow` = `["/etc/resolv.conf", "~/.gitconfig"]`.

---

### 3. Manifest field — `apm-core/src/wrapper/custom.rs`

Add to `Manifest`:

```rust
#[serde(default)]
pub enforce_worktree_isolation: bool,  // default false
```

Layer 1 (`validate_agents`): no change needed — unknown keys already emit a
warning; the new known key will simply be parsed.

Layer 2 (spawn time): in `CustomWrapper::spawn()`, after parsing the manifest,
if `manifest.enforce_worktree_isolation` is true, register `PathGuard` with the
interception hook from the wrapper epic.

---

### 4. Built-in claude wrapper — `apm-core/src/wrapper/builtin/claude.rs`

In `spawn_local()` (and `spawn_container()` if applicable):

- Check `ctx.skip_permissions` (the `-P` flag). When `true`, enforcement is
  **mandatory** regardless of any manifest field — this is the primary threat
  model.
- When `ctx.skip_permissions` is false, enforcement is opt-in (manifest field
  controls it; default false).
- In both cases, construct `PathGuard::new(&ctx, &cfg)` and register it with
  the epic's interception hook before spawning the claude process.

Concretely, the hook registration likely looks like (exact API TBD per epic):

```rust
let guard = PathGuard::new(&ctx, &isolation_cfg);
let hook = move |event: &ToolUseEvent| -> Result<(), String> {
    match event.name.as_str() {
        "Edit" => {
            let path = event.input["file_path"].as_str()?;
            guard.check_write(Path::new(path))
        }
        "Write" => {
            let path = event.input["file_path"].as_str()?;
            guard.check_write(Path::new(path))
        }
        "Bash" => {
            let cmd = event.input["command"].as_str()?;
            guard.check_bash(cmd)
        }
        _ => Ok(()),
    }
};
ctx_builder.set_tool_intercept(hook);  // or equivalent epic API
```

---

### 5. Integration tests — `apm-core/tests/path_guard_integration.rs`

One test per acceptance criterion that requires a subprocess. Pattern mirrors
`custom_wrapper_integration.rs`:

- Spawn a worker using the mock-happy or debug wrapper modified to emit
  specific `tool_use` events.
- Assert rejection or allowance based on the path.
- For the unmodified-file assertions, write a sentinel file in the main worktree
  before the test and assert it is unchanged after.

Unit tests in `path_guard.rs` cover:
- `check_write` with paths inside/outside the worktree
- `check_write` with symlink traversal
- `check_bash` with write redirects, read-only commands, mixed commands
- `canonicalize_lenient` with `..` escapes and non-existent paths

---

### Order of steps

1. Merge wrapper epic (4312fbd4) — provides `ToolUseEvent` type and
   `set_tool_intercept` hook.
2. Add `IsolationConfig` to `config.rs` and parse it in `ApmConfig::load()`.
3. Implement `path_guard.rs` with unit tests passing.
4. Add `enforce_worktree_isolation` to `Manifest`; wire into `CustomWrapper::spawn()`.
5. Wire into `ClaudeWrapper::spawn_local()` (mandatory for `-P`, opt-in otherwise).
6. Write integration tests; confirm all acceptance criteria pass.

### Open questions


### Amendment requests

- [ ] The wrapper epic does not expose a `set_tool_intercept` hook. The spec at §1/§4 assumes the canonical parser calls a registered callback per `tool_use` event, but reading `wrapper/mod.rs` and `ClaudeWrapper::spawn_local` shows the canonical parser is just stdout-tee'd to the log — there is no per-event interception layer. Either add ACs that build the hook in this ticket (more scope), or split out a prerequisite ticket. Without it, `claude` runs unsupervised and the rejection-injection back into the agent's tool stream cannot land.

- [ ] `-P` (`--dangerously-skip-permissions`) enforcement claim is unimplementable as written. The flag bypasses Claude Code's allowlist *inside* the claude binary; APM cannot intercept the inner tool dispatch unless it parses the JSONL stream and round-trips a synthetic `tool_result` — which requires an interactive `--input-format=stream-json` link, not the current one-shot `--print`. AC #1 needs to specify the IPC mode change, or the threat model needs to drop to "log-and-warn" rather than block.

- [ ] The Bash heuristic regex is too loose to be testable. AC says "embedded paths in subshells / variables not caught" is OOS, but the spec doesn't list the specific shapes that ARE caught. Add a small canonical table — at least 6 examples each of "this fires" and "this does not" — so the integration test set is bounded.

- [ ] `canonicalize_lenient` for write checks is racey. If the path doesn't exist yet, the validator resolves `..` lexically — but a parent symlink could redirect after the check. Add an AC: "intermediate components that exist must be canonicalised; the final non-existent leaf is appended after parent resolution."

- [ ] No AC verifies APM_BIN write protection works when APM_BIN is *under* the worktree (e.g. local cargo build placed in `target/`). Pin this edge case.

- [ ] `isolation.read_allow` glob support (e.g. `/etc/ssl/certs/**`) implies a glob crate dep. The spec does not pick one or specify semantics. Decide: literal-prefix match vs globset, and state which crate is added.

- [ ] After the above amendments, re-evaluate effort/risk. Effort 5 / risk 5 is plausible only if the wrapper-hook gap is split out into a prerequisite ticket; otherwise risk → 8.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T02:30Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:21Z | groomed | in_design | philippepascal |
| 2026-05-02T03:30Z | in_design | specd | claude-0502-0321-0790 |
| 2026-05-02T07:20Z | specd | ammend | claude-0502-1300-rev1 |
| 2026-05-02T08:02Z | ammend | in_design | philippepascal |