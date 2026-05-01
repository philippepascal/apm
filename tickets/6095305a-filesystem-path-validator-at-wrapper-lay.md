+++
id = "6095305a"
title = "Filesystem path validator at wrapper layer (worktree isolation enforcement)"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6095305a-filesystem-path-validator-at-wrapper-lay"
created_at = "2026-05-01T02:30:34.552318Z"
updated_at = "2026-05-01T02:30:34.552318Z"
+++

## Spec

### Problem

The wrapper layer (post-epic 4312fbd4) sees every tool call the underlying agent makes — that is the natural intercept point for filesystem isolation. This ticket adds a worktree-isolation enforcement: workers cannot make Edit/Write/Bash calls with absolute paths outside their assigned ticket worktree.

**Why this is needed even with the existing permission system:**
- Default-spawn workers get partial protection from Claude Code's permission allowlist — calls with paths not matching project allowlist patterns get denied.
- `-P` (`--dangerously-skip-permissions`) workers bypass the allowlist entirely. Today such a worker can write anywhere on the filesystem.
- The permission system catches paths it doesn't recognise; explicit project allowlist entries (e.g. for `apm spec` paths) accidentally cover legitimate ticket-worktree paths but also extend to main-worktree paths with the same prefix.

**Should land after the wrapper epic (4312fbd4) — the wrapper-layer interception is the implementation surface.**

**Scope:**
- In each shipped wrapper (claude built-in initially), parse the underlying agent's tool-call stream. For `Edit`, `Write`, and `Bash` invocations that reference filesystem paths:
  - Resolve the path to absolute, canonicalised form.
  - If it falls outside `APM_TICKET_WORKTREE`, intercept and reject before forwarding to the agent's tool dispatcher. The agent receives a tool-result error: "path outside ticket worktree; isolation enforced by APM wrapper. APM_TICKET_WORKTREE = <path>".
  - If it falls inside, allow.
- Allow-list of legitimate exceptions (paths the wrapper still permits writes/reads to):
  - `APM_BIN` itself (read for shellouts, never written).
  - The temp files APM provides via `APM_SYSTEM_PROMPT_FILE` and `APM_USER_MESSAGE_FILE`.
  - System paths the agent may legitimately need to read (e.g. `/etc/resolv.conf`, `~/.gitconfig`) — read-only, configurable.
- Custom wrappers opt in via a manifest field: `enforce_worktree_isolation = true` (default off for backward compat; flag in spec phase whether to flip the default).
- Bash calls are trickier — paths are inside the command string. Heuristic: parse the command for absolute paths and check those. False positives are fine (worse case, agent retries with a relative path); false negatives are the failure mode to avoid.

**Out of scope:**
- Process-level filesystem sandboxing (bwrap, sandbox-exec, containers). Heavier; only justified if this tool-level filter proves insufficient.
- Network egress filtering. The agent talks to the Anthropic API; that traffic is separate.
- Read-only filesystem access protections beyond the explicit allow-list. Reads outside the worktree are mostly information-only; the more important class is writes.

**Acceptance pointers:**
- Integration test: a wrapper running an agent that issues an Edit against `/Users/.../repos/apm/.apm/config.toml` (the main worktree) → call rejected, agent receives error, ticket worktree unmodified, main worktree unmodified.
- Integration test: the same Edit against a path inside `APM_TICKET_WORKTREE` → allowed.
- Integration test: a Bash call `cat /etc/resolv.conf` → allowed (in default read-allowlist).
- Integration test: a Bash call `echo foo > /tmp/leak` with `/tmp/*` not in allowlist → rejected.
- Per-wrapper opt-in respected: a custom wrapper with `enforce_worktree_isolation = false` does not intercept (backward compat).

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
| 2026-05-01T02:30Z | — | new | philippepascal |
