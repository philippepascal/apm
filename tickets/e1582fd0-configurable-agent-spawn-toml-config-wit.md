+++
id = "e1582fd0"
title = "Configurable agent spawn: TOML config with local overrides replaces hardcoded Command"
state = "in_design"
priority = 7
effort = 0
risk = 0
author = "apm"
branch = "ticket/e1582fd0-configurable-agent-spawn-toml-config-wit"
created_at = "2026-04-03T21:53:31.381487Z"
updated_at = "2026-04-03T21:54:24.258763Z"
+++

## Spec

### Problem

The worker spawn command is hardcoded in `apm-core/src/start.rs`. Three nearly identical blocks (in `run`, `run_next`, `spawn_next_worker`) each build `Command::new("claude")` with `--print`, `--system-prompt`, and optionally `--dangerously-skip-permissions`. Users cannot:

- Change the model (`--model opus`)
- Add extra CLI flags or env vars
- Swap `claude` for a different agent CLI (Codex, Aider, custom wrapper)
- Override per-machine without recompiling

The container path (`docker run ... claude`) has the same problem.

The fix is to move the spawn command definition into tracked TOML config (`[workers]` in `.apm/agents.toml` or `workflow.toml`) with per-machine overrides via a gitignored `local.toml`. apm reads the config and builds the `Command` at runtime — no shell scripts, no OS-specific files, cross-platform by default.

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
| 2026-04-03T21:53Z | — | new | apm |
| 2026-04-03T21:54Z | new | groomed | apm |
| 2026-04-03T21:54Z | groomed | in_design | apm |