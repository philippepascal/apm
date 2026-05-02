+++
id = "6095305a"
title = "Filesystem path validator at wrapper layer (worktree isolation enforcement)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6095305a-filesystem-path-validator-at-wrapper-lay"
created_at = "2026-05-01T02:30:34.552318Z"
updated_at = "2026-05-02T03:21:24.434090Z"
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

The wrapper epic (4312fbd4) introduces an interception layer that sees every
tool-call event before it is dispatched. That is the correct surface for
enforcement: check the target path before the tool executes, and inject a
synthetic `tool_result` error if the path violates policy.

This ticket wires a `PathGuard` into that interception hook for the
`claude` built-in wrapper, adds a per-manifest opt-in field for custom
wrappers, and backs the allow-list with a project-level config section.

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

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T02:30Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:21Z | groomed | in_design | philippepascal |