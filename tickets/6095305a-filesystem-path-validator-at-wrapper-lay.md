+++
id = "6095305a"
title = "Filesystem path validator at wrapper layer (worktree isolation enforcement)"
state = "closed"
priority = 0
effort = 6
risk = 6
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6095305a-filesystem-path-validator-at-wrapper-lay"
created_at = "2026-05-01T02:30:34.552318Z"
updated_at = "2026-05-03T20:15:06.340688Z"
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

- [x] `apm start` with `enforce_worktree_isolation = true` spawns the worker with path enforcement active; a worker that issues `Edit` against a path in the main worktree receives a `tool_result` error whose message contains "path outside ticket worktree"
- [x] The rejection `tool_result` message includes the value of `APM_TICKET_WORKTREE` so the agent can self-correct
- [x] The main-worktree file targeted by the rejected `Edit` call is unmodified after the rejection
- [x] A worker issues `Edit` against a path inside `APM_TICKET_WORKTREE`; the call succeeds and the file is modified
- [x] A worker issues `Write` against a path outside `APM_TICKET_WORKTREE`; the call is rejected with the same error format
- [x] A worker issues `Bash` with command `echo foo > /path/outside/worktree`; the command is rejected before execution; the target file is unmodified
- [x] A worker issues `Bash` with command `cat /etc/resolv.conf`; the call is allowed (default read-allow-list entry)
- [x] A worker issues `Bash` with command `cat ~/.gitconfig`; the call is allowed (default read-allow-list entry)
- [x] A worker issues `Bash` whose only absolute paths are inside `APM_TICKET_WORKTREE`; the call is allowed
- [x] A custom wrapper with `enforce_worktree_isolation = false` in its `manifest.toml` runs without path interception; the worker can write outside `APM_TICKET_WORKTREE` unobstructed
- [x] A custom wrapper whose `manifest.toml` omits `enforce_worktree_isolation` behaves identically to `false` (opt-in, backward-compatible default)
- [x] Path resolution canonicalises `..` components before comparison; a path like `<worktree>/../../../etc/passwd` is rejected
- [x] Path resolution follows symlinks before comparison; a symlink inside `APM_TICKET_WORKTREE` that resolves outside it is rejected
- [x] A `Write` call targeting `APM_BIN` is rejected even when `APM_BIN` has no path relationship to the worktree
- [x] A `Write` call targeting `APM_SYSTEM_PROMPT_FILE` or `APM_USER_MESSAGE_FILE` is rejected (those paths are read-only exceptions, not writable)
- [x] The read-allow-list is configurable in `.apm/config.toml` under `[isolation] read_allow`; entries added there permit the corresponding `Bash cat` calls through enforcement
- [x] A worker spawned with `-P` (`--dangerously-skip-permissions`) and `enforce_worktree_isolation = true` that issues `Edit` against the main worktree receives the same `tool_result` error as a non-`-P` worker; the `PreToolUse` hook fires regardless of the skip-permissions flag
- [x] When a write target path does not yet exist, `PathGuard` canonicalises all existing ancestor components by following symlinks, then appends the non-existent filename lexically; a path `<worktree>/subdir/../../etc/passwd` where `<worktree>/subdir` exists as a real directory is rejected
- [x] A `Write` call targeting a path that canonicalises to `APM_BIN` is rejected even when `APM_BIN` happens to reside inside `APM_TICKET_WORKTREE` (e.g. a local Cargo build at `target/debug/apm`)

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

#### Overview

Enforcement uses Claude Code's `PreToolUse` hook API. Before spawning the worker
process, APM writes a hook entry to `<worktree>/.claude/settings.json`. Claude
Code invokes this hook before every tool call; the hook runs `apm path-guard`,
which reads the tool name and input from stdin, evaluates `PathGuard`, and exits
0 (allow) or 2 (reject, message on stdout). Claude Code converts exit-2 into a
synthetic `tool_result` error sent back to the model. The hook fires regardless
of `--dangerously-skip-permissions` -- that flag disables interactive permission
prompts, not hooks. No wrapper epic or external IPC mechanism is required; this
ticket is self-contained.

---

#### 1. `PathGuard` -- `apm-core/src/wrapper/path_guard.rs`

```rust
pub struct PathGuard {
    worktree: PathBuf,             // canonicalised APM_TICKET_WORKTREE
    read_allow: globset::GlobSet,  // compiled patterns for read-only Bash cmds
    write_protected: Vec<PathBuf>, // APM_BIN, APM_SYSTEM_PROMPT_FILE, APM_USER_MESSAGE_FILE
}
```

Key functions:

- `PathGuard::new(worktree: &Path, read_allow_patterns: &[String], write_protected: &[PathBuf]) -> anyhow::Result<Self>`
  Canonicalises `worktree`. Expands `~` in each pattern to `$HOME`, then compiles
  into a `globset::GlobSet` (crate `globset = "0.4"`, added to
  `apm-core/Cargo.toml`). Populates `write_protected` from the provided paths.

- `PathGuard::check_write(&self, path: &Path) -> Result<(), String>`
  1. Resolve: `canonicalize_lenient(path)`.
  2. If resolved starts with `self.worktree` AND is not in `self.write_protected`
     -> `Ok(())`.
  3. Otherwise -> `Err(rejection_msg)`. (`write_protected` entries are rejected
     even when they happen to be inside the worktree, e.g. `target/debug/apm`.)

- `PathGuard::check_bash(&self, cmd: &str) -> Result<(), String>`
  Applies write-detection heuristic (see canonical table). For each detected
  write-target path, calls `check_write`. Returns the first rejection or `Ok(())`.

- `canonicalize_lenient(path: &Path) -> PathBuf`
  Walk path components from root. For each prefix that exists, call
  `std::fs::canonicalize()` to follow symlinks. For components that do not yet
  exist, append them lexically (no syscall). This ensures existing intermediate
  symlinks are resolved while non-existent leaf paths are handled without TOCTOU.

**Bash write-target detection -- canonical table**

Token regex finds tokens starting with `/` or `~/` in the command string.
Classification of detected tokens:

| Detected as write target -- check_write fires | Trigger |
|----------------------------------------------|---------|
| `echo foo > /outside/file` | `>` redirect target |
| `cat data >> /outside/append.log` | `>>` redirect target |
| `tee /outside/output.txt` | `tee` first non-flag arg |
| `cp /inside/src /outside/dest` | `cp` destination (last operand) |
| `mv /inside/file /outside/dest` | `mv` destination (last operand) |
| `truncate -s 0 /outside/file` | `truncate` path operand |

| Not detected as write target -- check_write does not fire | Reason |
|----------------------------------------------------------|--------|
| `cat /etc/resolv.conf` | read-only cat, no redirect |
| `grep pattern /etc/hosts` | grep reads only |
| `ls /outside/dir` | listing, no write |
| `diff /file1 /file2` | comparison, no write |
| `wc -l /var/log/syslog` | read-only word count |
| `echo hello` | no absolute path |

Known false negatives (documented limitation, not in scope):

| Command | Why missed |
|---------|-----------|
| `OUT=/outside/file; echo foo > "$OUT"` | variable interpolation |
| `echo foo > $(cat /tmp/path)` | subshell expansion |
| `eval "echo foo > /outside/file"` | eval |

**Rejection message template**:

```
path outside ticket worktree; isolation enforced by APM wrapper.
  Requested: {requested_path}
  APM_TICKET_WORKTREE = {worktree}
```

---

#### 2. `IsolationConfig` -- `apm-core/src/config.rs`

```rust
#[derive(Debug, Clone, Deserialize, Default, JsonSchema)]
pub struct IsolationConfig {
    #[serde(default)]
    pub read_allow: Vec<String>,   // globset patterns; ~ expanded before compilation
}
```

Add `pub isolation: IsolationConfig` to `ApmConfig` with `#[serde(default)]`.

Example `.apm/config.toml`:

```toml
[isolation]
read_allow = [
  "/etc/resolv.conf",
  "~/.gitconfig",
  "~/.ssh/config",
  "/etc/ssl/certs/**",
]
```

Default when `[isolation]` is absent: `read_allow = ["/etc/resolv.conf", "~/.gitconfig"]`.

**Glob semantics**: `globset` crate (`globset = "0.4"` in `apm-core/Cargo.toml`).
`~` expanded to `$HOME` before `GlobSet::build()`. `**` matches zero or more path
components. Literal patterns (no wildcards) match exactly.

---

#### 3. Manifest field -- `apm-core/src/wrapper/custom.rs`

Add to `Manifest`:

```rust
#[serde(default)]
pub enforce_worktree_isolation: bool,  // default false
```

Add `"enforce_worktree_isolation"` to the `known` array in `manifest_unknown_keys()`.

In `CustomWrapper::spawn()`, after contract-version check: if
`manifest.enforce_worktree_isolation` is true, call `write_hook_config` (SS4)
before spawning. Wrappers with `parser = "external"` are exempt (out of scope).

---

#### 4. Hook configuration -- `apm-core/src/wrapper/hook_config.rs`

New module.

```rust
pub fn write_hook_config(worktree: &Path, apm_bin: &str) -> anyhow::Result<()>
pub fn remove_hook_config(worktree: &Path) -> anyhow::Result<()>
```

`write_hook_config`:

1. Path: `<worktree>/.claude/settings.json`. Create `.claude/` if absent.
2. Read and parse as `serde_json::Value`; default to `{}` if file missing.
3. Navigate to `hooks.PreToolUse` (create as JSON array if absent).
4. If no entry already has `command` ending in `"apm path-guard"`, append:
   ```json
   {
     "matcher": "Edit|Write|Bash",
     "hooks": [{"type": "command", "command": "<apm_bin> path-guard"}]
   }
   ```
5. Write back (pretty-printed JSON).

`remove_hook_config`: re-read the file, filter out any hook entry whose nested
`command` ends with `"apm path-guard"`, write back. Called after the child
process exits (or on spawn failure) to avoid leaving stale hooks in long-lived
worktrees. `<worktree>/.claude/settings.json` is not git-tracked (`.gitignore`
covers `.claude/`), so writes do not pollute the ticket branch.

---

#### 5. `apm path-guard` subcommand -- `apm/src/cmd/path_guard.rs`

New CLI subcommand. The hook command written to settings.json is
`<apm_bin> path-guard`.

**Stdin** (Claude Code PreToolUse hook contract):

```json
{"tool_name": "Edit", "tool_input": {"file_path": "/some/path"}}
```

**Environment** (already set by APM before spawning the worker):

- `APM_TICKET_WORKTREE` -- worktree root
- `APM_SYSTEM_PROMPT_FILE`, `APM_USER_MESSAGE_FILE` -- write-protected paths
- `APM_BIN` -- write-protected (the apm binary itself)

**Logic**:

1. Parse JSON from stdin. On parse failure -> exit 0 (do not block on malformed).
2. Walk upward from `APM_TICKET_WORKTREE` until `.apm/config.toml` found; load
   `IsolationConfig` (or use defaults).
3. Build `PathGuard` from env vars + config.
4. Dispatch on `tool_name`:
   - `"Edit"` or `"Write"`: extract `tool_input.file_path`, call `check_write`
   - `"Bash"`: extract `tool_input.command`, call `check_bash`
   - anything else: exit 0
5. `Ok(())` -> exit 0; `Err(msg)` -> print msg to stdout, exit 2

---

#### 6. Built-in claude wrapper -- `apm-core/src/wrapper/builtin/claude.rs`

In `spawn_local()`:

- If `ctx.skip_permissions` is `true`: call `write_hook_config` unconditionally
  (mandatory enforcement for `-P` workers).
- Else if `cfg.isolation.enforce_worktree_isolation` is `true`: call
  `write_hook_config`.
- After child exits: call `remove_hook_config`.

Apply the same logic in `spawn_container()` -- hook config must be written before
the container mounts the worktree volume.

---

#### 7. Integration tests -- `apm-core/tests/path_guard_integration.rs`

Most ACs are exercised by invoking `apm path-guard` as a subprocess with crafted
JSON on stdin and env vars set in the test harness -- no full claude worker spawn
needed. Pattern:

```rust
let output = Command::new(apm_bin())
    .arg("path-guard")
    .env("APM_TICKET_WORKTREE", &worktree)
    .env("APM_BIN", apm_bin())
    .env("APM_SYSTEM_PROMPT_FILE", &sys_file)
    .env("APM_USER_MESSAGE_FILE", &msg_file)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    // ... pipe payload, then collect output
    .output()?;
assert_eq!(output.status.code(), Some(2));
assert!(String::from_utf8_lossy(&output.stdout).contains("path outside ticket worktree"));
```

One end-to-end test (gated `#[ignore]` if requiring the real `claude` binary):
write a sentinel file in the main worktree, spawn the worker with the mock-happy
wrapper and a fabricated tool_use event targeting that file, assert it is
unmodified after the worker exits.

Unit tests in `path_guard.rs` cover:

- `check_write` paths inside/outside worktree, symlink traversal, `write_protected` matches
- Each row of the canonical bash table (fires and does-not-fire cases)
- `canonicalize_lenient` with `..` escapes, symlinks in intermediate components, non-existent leaves
- APM_BIN-under-worktree edge case

---

#### Order of steps

1. Add `globset = "0.4"` to `apm-core/Cargo.toml`.
2. Add `IsolationConfig` to `config.rs`; parse in `ApmConfig::load()`; update `JsonSchema` derive.
3. Implement `path_guard.rs` with unit tests passing.
4. Implement `hook_config.rs` (`write_hook_config`, `remove_hook_config`).
5. Implement `apm path-guard` subcommand in `apm/src/cmd/path_guard.rs`; wire into CLI.
6. Add `enforce_worktree_isolation` to `Manifest`; update `manifest_unknown_keys`; wire `write_hook_config` into `CustomWrapper::spawn()`.
7. Wire into `ClaudeWrapper::spawn_local()` (mandatory for `-P`, opt-in otherwise) and `spawn_container()`.
8. Write integration tests; confirm all acceptance criteria pass.

No wrapper-epic merge is required before starting; this ticket is fully self-contained.

### Overview

Enforcement uses Claude Code's `PreToolUse` hook API. Before spawning the worker
process, APM writes a hook entry to `<worktree>/.claude/settings.json`. Claude
Code invokes this hook before every tool call; the hook runs `apm path-guard`,
which reads the tool name and input from stdin, evaluates `PathGuard`, and exits
0 (allow) or 2 (reject, message on stdout). Claude Code converts exit-2 into a
synthetic `tool_result` error sent back to the model. The hook fires regardless
of `--dangerously-skip-permissions` -- that flag disables interactive permission
prompts, not hooks. No wrapper epic or external IPC mechanism is required; this
ticket is self-contained.

---

### 1. `PathGuard` -- `apm-core/src/wrapper/path_guard.rs`

```rust
pub struct PathGuard {
    worktree: PathBuf,             // canonicalised APM_TICKET_WORKTREE
    read_allow: globset::GlobSet,  // compiled patterns for read-only Bash cmds
    write_protected: Vec<PathBuf>, // APM_BIN, APM_SYSTEM_PROMPT_FILE, APM_USER_MESSAGE_FILE
}
```

Key functions:

- `PathGuard::new(worktree: &Path, read_allow_patterns: &[String], write_protected: &[PathBuf]) -> anyhow::Result<Self>`
  Canonicalises `worktree`. Expands `~` in each pattern to `$HOME`, then compiles
  into a `globset::GlobSet` (crate `globset = "0.4"`, added to
  `apm-core/Cargo.toml`). Populates `write_protected` from the provided paths.

- `PathGuard::check_write(&self, path: &Path) -> Result<(), String>`
  1. Resolve: `canonicalize_lenient(path)`.
  2. If resolved starts with `self.worktree` AND is not in `self.write_protected`
     -> `Ok(())`.
  3. Otherwise -> `Err(rejection_msg)`. (`write_protected` entries are rejected
     even when they happen to be inside the worktree, e.g. `target/debug/apm`.)

- `PathGuard::check_bash(&self, cmd: &str) -> Result<(), String>`
  Applies write-detection heuristic (see canonical table). For each detected
  write-target path, calls `check_write`. Returns the first rejection or `Ok(())`.

- `canonicalize_lenient(path: &Path) -> PathBuf`
  Walk path components from root. For each prefix that exists, call
  `std::fs::canonicalize()` to follow symlinks. For components that do not yet
  exist, append them lexically (no syscall). This ensures existing intermediate
  symlinks are resolved while non-existent leaf paths are handled without TOCTOU.

**Bash write-target detection -- canonical table**

Token regex finds tokens starting with `/` or `~/` in the command string.
Classification of detected tokens:

| Detected as write target -- check_write fires | Trigger |
|----------------------------------------------|---------|
| `echo foo > /outside/file` | `>` redirect target |
| `cat data >> /outside/append.log` | `>>` redirect target |
| `tee /outside/output.txt` | `tee` first non-flag arg |
| `cp /inside/src /outside/dest` | `cp` destination (last operand) |
| `mv /inside/file /outside/dest` | `mv` destination (last operand) |
| `truncate -s 0 /outside/file` | `truncate` path operand |

| Not detected as write target -- check_write does not fire | Reason |
|----------------------------------------------------------|--------|
| `cat /etc/resolv.conf` | read-only cat, no redirect |
| `grep pattern /etc/hosts` | grep reads only |
| `ls /outside/dir` | listing, no write |
| `diff /file1 /file2` | comparison, no write |
| `wc -l /var/log/syslog` | read-only word count |
| `echo hello` | no absolute path |

Known false negatives (documented limitation, not in scope):

| Command | Why missed |
|---------|-----------|
| `OUT=/outside/file; echo foo > "$OUT"` | variable interpolation |
| `echo foo > $(cat /tmp/path)` | subshell expansion |
| `eval "echo foo > /outside/file"` | eval |

**Rejection message template**:

```
path outside ticket worktree; isolation enforced by APM wrapper.
  Requested: {requested_path}
  APM_TICKET_WORKTREE = {worktree}
```

---

### 2. `IsolationConfig` -- `apm-core/src/config.rs`

```rust
#[derive(Debug, Clone, Deserialize, Default, JsonSchema)]
pub struct IsolationConfig {
    #[serde(default)]
    pub read_allow: Vec<String>,   // globset patterns; ~ expanded before compilation
}
```

Add `pub isolation: IsolationConfig` to `ApmConfig` with `#[serde(default)]`.

Example `.apm/config.toml`:

```toml
[isolation]
read_allow = [
  "/etc/resolv.conf",
  "~/.gitconfig",
  "~/.ssh/config",
  "/etc/ssl/certs/**",
]
```

Default when `[isolation]` is absent: `read_allow = ["/etc/resolv.conf", "~/.gitconfig"]`.

**Glob semantics**: `globset` crate (`globset = "0.4"` in `apm-core/Cargo.toml`).
`~` expanded to `$HOME` before `GlobSet::build()`. `**` matches zero or more path
components. Literal patterns (no wildcards) match exactly.

---

### 3. Manifest field -- `apm-core/src/wrapper/custom.rs`

Add to `Manifest`:

```rust
#[serde(default)]
pub enforce_worktree_isolation: bool,  // default false
```

Add `"enforce_worktree_isolation"` to the `known` array in `manifest_unknown_keys()`.

In `CustomWrapper::spawn()`, after contract-version check: if
`manifest.enforce_worktree_isolation` is true, call `write_hook_config` (SS4)
before spawning. Wrappers with `parser = "external"` are exempt (out of scope).

---

### 4. Hook configuration -- `apm-core/src/wrapper/hook_config.rs`

New module.

```rust
pub fn write_hook_config(worktree: &Path, apm_bin: &str) -> anyhow::Result<()>
pub fn remove_hook_config(worktree: &Path) -> anyhow::Result<()>
```

`write_hook_config`:

1. Path: `<worktree>/.claude/settings.json`. Create `.claude/` if absent.
2. Read and parse as `serde_json::Value`; default to `{}` if file missing.
3. Navigate to `hooks.PreToolUse` (create as JSON array if absent).
4. If no entry already has `command` ending in `"apm path-guard"`, append:
   ```json
   {
     "matcher": "Edit|Write|Bash",
     "hooks": [{"type": "command", "command": "<apm_bin> path-guard"}]
   }
   ```
5. Write back (pretty-printed JSON).

`remove_hook_config`: re-read the file, filter out any hook entry whose nested
`command` ends with `"apm path-guard"`, write back. Called after the child
process exits (or on spawn failure) to avoid leaving stale hooks in long-lived
worktrees. `<worktree>/.claude/settings.json` is not git-tracked (`.gitignore`
covers `.claude/`), so writes do not pollute the ticket branch.

---

### 5. `apm path-guard` subcommand -- `apm/src/cmd/path_guard.rs`

New CLI subcommand. The hook command written to settings.json is
`<apm_bin> path-guard`.

**Stdin** (Claude Code PreToolUse hook contract):

```json
{"tool_name": "Edit", "tool_input": {"file_path": "/some/path"}}
```

**Environment** (already set by APM before spawning the worker):

- `APM_TICKET_WORKTREE` -- worktree root
- `APM_SYSTEM_PROMPT_FILE`, `APM_USER_MESSAGE_FILE` -- write-protected paths
- `APM_BIN` -- write-protected (the apm binary itself)

**Logic**:

1. Parse JSON from stdin. On parse failure -> exit 0 (do not block on malformed).
2. Walk upward from `APM_TICKET_WORKTREE` until `.apm/config.toml` found; load
   `IsolationConfig` (or use defaults).
3. Build `PathGuard` from env vars + config.
4. Dispatch on `tool_name`:
   - `"Edit"` or `"Write"`: extract `tool_input.file_path`, call `check_write`
   - `"Bash"`: extract `tool_input.command`, call `check_bash`
   - anything else: exit 0
5. `Ok(())` -> exit 0; `Err(msg)` -> print msg to stdout, exit 2

---

### 6. Built-in claude wrapper -- `apm-core/src/wrapper/builtin/claude.rs`

In `spawn_local()`:

- If `ctx.skip_permissions` is `true`: call `write_hook_config` unconditionally
  (mandatory enforcement for `-P` workers).
- Else if `cfg.isolation.enforce_worktree_isolation` is `true`: call
  `write_hook_config`.
- After child exits: call `remove_hook_config`.

Apply the same logic in `spawn_container()` -- hook config must be written before
the container mounts the worktree volume.

---

### 7. Integration tests -- `apm-core/tests/path_guard_integration.rs`

Most ACs are exercised by invoking `apm path-guard` as a subprocess with crafted
JSON on stdin and env vars set in the test harness -- no full claude worker spawn
needed. Pattern:

```rust
let output = Command::new(apm_bin())
    .arg("path-guard")
    .env("APM_TICKET_WORKTREE", &worktree)
    .env("APM_BIN", apm_bin())
    .env("APM_SYSTEM_PROMPT_FILE", &sys_file)
    .env("APM_USER_MESSAGE_FILE", &msg_file)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    // ... pipe payload, then collect output
    .output()?;
assert_eq!(output.status.code(), Some(2));
assert!(String::from_utf8_lossy(&output.stdout).contains("path outside ticket worktree"));
```

One end-to-end test (gated `#[ignore]` if requiring the real `claude` binary):
write a sentinel file in the main worktree, spawn the worker with the mock-happy
wrapper and a fabricated tool_use event targeting that file, assert it is
unmodified after the worker exits.

Unit tests in `path_guard.rs` cover:

- `check_write` paths inside/outside worktree, symlink traversal, `write_protected` matches
- Each row of the canonical bash table (fires and does-not-fire cases)
- `canonicalize_lenient` with `..` escapes, symlinks in intermediate components, non-existent leaves
- APM_BIN-under-worktree edge case

---

### Order of steps

1. Merge wrapper epic (4312fbd4) — provides `ToolUseEvent` type and
   `set_tool_intercept` hook.
2. Add `IsolationConfig` to `config.rs` and parse it in `ApmConfig::load()`.
3. Implement `path_guard.rs` with unit tests passing.
4. Add `enforce_worktree_isolation` to `Manifest`; wire into `CustomWrapper::spawn()`.
5. Wire into `ClaudeWrapper::spawn_local()` (mandatory for `-P`, opt-in otherwise).
6. Write integration tests; confirm all acceptance criteria pass.

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

### Open questions


### Amendment requests

- [x] The wrapper epic does not expose a `set_tool_intercept` hook. The spec at §1/§4 assumes the canonical parser calls a registered callback per `tool_use` event, but reading `wrapper/mod.rs` and `ClaudeWrapper::spawn_local` shows the canonical parser is just stdout-tee'd to the log — there is no per-event interception layer. Either add ACs that build the hook in this ticket (more scope), or split out a prerequisite ticket. Without it, `claude` runs unsupervised and the rejection-injection back into the agent's tool stream cannot land.

- [x] `-P` (`--dangerously-skip-permissions`) enforcement claim is unimplementable as written. The flag bypasses Claude Code's allowlist *inside* the claude binary; APM cannot intercept the inner tool dispatch unless it parses the JSONL stream and round-trips a synthetic `tool_result` — which requires an interactive `--input-format=stream-json` link, not the current one-shot `--print`. AC #1 needs to specify the IPC mode change, or the threat model needs to drop to "log-and-warn" rather than block.

- [x] The Bash heuristic regex is too loose to be testable. AC says "embedded paths in subshells / variables not caught" is OOS, but the spec doesn't list the specific shapes that ARE caught. Add a small canonical table — at least 6 examples each of "this fires" and "this does not" — so the integration test set is bounded.

- [x] `canonicalize_lenient` for write checks is racey. If the path doesn't exist yet, the validator resolves `..` lexically — but a parent symlink could redirect after the check. Add an AC: "intermediate components that exist must be canonicalised; the final non-existent leaf is appended after parent resolution."

- [x] No AC verifies APM_BIN write protection works when APM_BIN is *under* the worktree (e.g. local cargo build placed in `target/`). Pin this edge case.

- [x] `isolation.read_allow` glob support (e.g. `/etc/ssl/certs/**`) implies a glob crate dep. The spec does not pick one or specify semantics. Decide: literal-prefix match vs globset, and state which crate is added.

- [x] After the above amendments, re-evaluate effort/risk. Effort 5 / risk 5 is plausible only if the wrapper-hook gap is split out into a prerequisite ticket; otherwise risk → 8.

### Code review


### Merge notes

merge conflict — resolve manually and push: 

## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T02:30Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:21Z | groomed | in_design | philippepascal |
| 2026-05-02T03:30Z | in_design | specd | claude-0502-0321-0790 |
| 2026-05-02T07:20Z | specd | ammend | claude-0502-1300-rev1 |
| 2026-05-02T08:02Z | ammend | in_design | philippepascal |
| 2026-05-02T08:18Z | in_design | specd | claude-0502-0802-ac68 |
| 2026-05-02T18:21Z | specd | ready | philippepascal |
| 2026-05-02T19:22Z | ready | in_progress | philippepascal |
| 2026-05-02T19:46Z | in_progress | implemented | claude-0502-1922-24a8 |
| 2026-05-02T19:46Z | implemented | merge_failed | claude-0502-1922-24a8 |
| 2026-05-03T07:53Z | merge_failed | implemented | philippepascal |
| 2026-05-03T07:57Z | implemented | in_progress | philippepascal |
| 2026-05-03T07:57Z | in_progress | implemented | philippepascal |
| 2026-05-03T07:57Z | implemented | merge_failed | philippepascal |
| 2026-05-03T07:59Z | merge_failed | implemented | philippepascal |
| 2026-05-03T08:00Z | implemented | in_progress | philippepascal |
| 2026-05-03T08:00Z | in_progress | implemented | philippepascal |
| 2026-05-03T08:00Z | implemented | merge_failed | philippepascal |
| 2026-05-03T08:06Z | merge_failed | implemented | philippepascal |
| 2026-05-03T20:15Z | implemented | closed | philippepascal |
