+++
id = "daf83745"
title = "Rename worker profile to coder across defaults and live config"
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/daf83745-rename-worker-profile-to-coder-across-de"
created_at = "2026-05-25T00:45:42.028363Z"
updated_at = "2026-05-25T01:25:57.198517Z"
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

- Renaming the TOML key `worker_profile` itself — it's a config field name, not a role name
- Renaming Rust structs and types that contain "worker" (e.g., `ResolvedWorkerProfile`, `WorkerProfileManifest`, `WorkersConfig`)
- Adding a backward-compatibility fallback for projects with `"claude/worker"` in their existing config
- Updating documentation files (README.md, docs/)
- Updating other tickets whose spec text references `"claude/worker"` as an example

### Approach

This is a mechanical string-replacement and file-rename across source code, tests, and live config. No logic changes.

#### 1. Built-in default files — git mv (five files)

Use `git mv` to rename each file (contents unchanged):
- `apm-core/src/default/agents/claude/apm.worker.md` → `apm.coder.md`
- `apm-core/src/default/agents/mock-happy/apm.worker.md` → `apm.coder.md`
- `apm-core/src/default/agents/mock-sad/apm.worker.md` → `apm.coder.md`
- `apm-core/src/default/agents/mock-random/apm.worker.md` → `apm.coder.md`
- `apm-core/src/default/agents/debug/apm.worker.md` → `apm.coder.md`

#### 2. apm-core/src/default/workflow.toml

Line 135: `worker_profile = "claude/worker"` → `worker_profile = "claude/coder"`.

#### 3. apm-core/src/start.rs

a) **const declarations (lines 7–16)** — rename the five `*_WORKER_DEFAULT` consts to `*_CODER_DEFAULT` and update each `include_str!` path from `apm.worker.md` to `apm.coder.md`.

b) **`resolve_builtin_instructions` match arms** — change the role string from `"worker"` to `"coder"` in all five match arms (`claude`, `default`, `mock-happy`, `mock-sad`, `mock-random`, `mock-random`, `debug`), and update the referenced const names to match step (a).

c) **Three `.unwrap_or` fallbacks** (lines 321, 449, 626): `"claude/worker"` → `"claude/coder"`.

d) **Tests** — update:
- All `"claude/worker"` profile strings passed to `resolve_worker_profile` → `"claude/coder"`
- All `apm.worker.md` path strings in test file writes → `apm.coder.md`
- All `worker.toml` manifest file names in tests → `coder.toml` (the profile manifest is named after the role, so `coder.toml` for `claude/coder`)

#### 4. apm-core/src/init.rs

a) Two `.unwrap_or("claude/worker")` calls (lines 87 and 118) → `.unwrap_or("claude/coder")`.

b) The `write_default` call at line 131 that writes `apm.worker.md`: change the path argument and the label string to `apm.coder.md`.

c) **Tests**: update `default_config(..., "claude/worker")` calls to `"claude/coder"` and the `apm.worker.md` file-existence assert (line 735) to `apm.coder.md`.

#### 5. apm-core/src/validate.rs

Line 722: `.unwrap_or("claude/worker")` → `.unwrap_or("claude/coder")`.

#### 6. apm-core/src/config.rs

- Line 112: doc comment example `"claude/worker"` → `"claude/coder"`.
- Lines 932 and 935: inline test TOML and assertion → `"claude/coder"`.

#### 7. apm-core/src/instructions.rs

Lines 598 and 608: `worker_profile = "claude/worker"` → `"claude/coder"` in the example TOML embedded in generated instructions output.

#### 8. apm-core/src/prompt.rs

- Line 68: `.unwrap_or("claude/worker")` → `.unwrap_or("claude/coder")`.
- Lines 572–573: update test assertion strings from `"claude/worker"` / `apm.worker.md` to `"claude/coder"` / `apm.coder.md`.

#### 9. apm/src/main.rs

Lines 903–904: update the `--role worker` help text example to `--role coder`.

#### 10. apm/tests/validate_fix.rs

- Lines 52, 136: assertion `Some("claude/worker")` → `Some("claude/coder")`.
- Line 151: embedded TOML `default = "claude/worker"` → `"claude/coder"`.

#### 11. apm/tests/integration.rs

- Line 1642: `replace("default = \"claude/worker\"", "default = \"mock-happy/worker\"")` → `replace("default = \"claude/coder\"", "default = \"mock-happy/coder\"")`.
- Line 1988: same replacement.

#### 12. Live project files

- `.apm/config.toml`: `default = "claude/worker"` → `default = "claude/coder"`.
- `.apm/workflow.toml`: `worker_profile = "claude/worker"` → `worker_profile = "claude/coder"`.
- `git mv .apm/agents/claude/apm.worker.md .apm/agents/claude/apm.coder.md`.

Commit all source, test, and live-config changes together in a single commit on the ticket branch.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-25T00:45Z | — | new | philippepascal |
| 2026-05-25T01:19Z | new | groomed | philippepascal |
| 2026-05-25T01:20Z | groomed | in_design | philippepascal |