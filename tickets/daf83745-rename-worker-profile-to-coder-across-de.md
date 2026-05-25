+++
id = "daf83745"
title = "Rename worker profile to coder across defaults and live config"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/daf83745-rename-worker-profile-to-coder-across-de"
created_at = "2026-05-25T00:45:42.028363Z"
updated_at = "2026-05-25T01:20:01.848389Z"
+++

## Spec

### Problem

The role name 'worker' in the 'claude/worker' profile string is generic and doesn't communicate what the role does. 'coder' is more descriptive — it makes the profile's purpose immediately clear to someone reading the config.

This rename touches every place the string 'worker' appears as a role name (not a TOML key):

**apm-core/src/start.rs**
The hardcoded fallback at the end of the worker_profile resolution chain:
  .unwrap_or("claude/worker")
in run(), run_next(), and spawn_next_worker() — all three must change to "claude/coder".

**apm-core/src/init.rs — default_config()**
The generated config.toml template contains:
  default = "claude/worker"
This must become:
  default = "claude/coder"

The generated workflow.toml template (via init_workflow_config() or equivalent) contains worker_profile = "claude/worker" on the transitions that use it. These must become "claude/coder".

**apm-core/src/init.rs — default_workflow_config() (or whichever function writes workflow.toml)**
Same as above — all worker_profile = "claude/worker" entries become "claude/coder".

**Live .apm/config.toml**
  default = "claude/worker"  →  default = "claude/coder"

**Live .apm/workflow.toml**
All occurrences of worker_profile = "claude/worker" → worker_profile = "claude/coder".

**Live .apm/agents/claude/**
Rename apm.worker.md to apm.coder.md. File contents remain unchanged.

**apm/tests/integration.rs**
setup_with_local_worktrees() and setup_for_prompt_dispatch() patch "claude/worker" in config.toml and "claude/" in workflow.toml to replace with mock-happy equivalents. These string replacements target the config values being patched so they must be updated to match the new "claude/coder" key.

### Acceptance criteria

- [ ] `apm start` and `apm start --next` fall back to `claude/coder` (not `claude/worker`) when no `worker_profile` is set on the transition and no `default` is set in `[workers]`
- [ ] `apm init` generates `.apm/config.toml` with `default = "claude/coder"` in `[workers]`
- [ ] `apm init` generates `.apm/workflow.toml` with `worker_profile = "claude/coder"` on the `ready → in_progress` transition
- [ ] `apm init` writes `.apm/agents/claude/apm.coder.md` (not `apm.worker.md`)
- [ ] The built-in instruction cascade resolves `claude/coder`, `mock-happy/coder`, `mock-sad/coder`, `mock-random/coder`, and `debug/coder` without error
- [ ] The live `.apm/config.toml` contains `default = "claude/coder"`
- [ ] The live `.apm/workflow.toml` contains `worker_profile = "claude/coder"` on the `ready → in_progress` transition
- [ ] The live `.apm/agents/claude/` directory contains `apm.coder.md` and no `apm.worker.md`
- [ ] `cargo test --workspace` passes with all assertions updated to `"claude/coder"`

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
| 2026-05-25T00:45Z | — | new | philippepascal |
| 2026-05-25T01:19Z | new | groomed | philippepascal |
| 2026-05-25T01:20Z | groomed | in_design | philippepascal |