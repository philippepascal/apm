+++
id = "d8e2fa0e"
title = "Redesign build_system_prompt to compose three layers"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/d8e2fa0e-redesign-build-system-prompt-to-compose-"
created_at = "2026-05-22T23:23:06.850140Z"
updated_at = "2026-05-23T00:20:56.556438Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
depends_on = ["4bee5771", "edb0cf35"]
+++

## Spec

### Problem

`build_system_prompt` (apm-core/src/start.rs) currently works as: prepend the file at `config.agents.instructions` → then pick a single cascade winner from the role-file cascade (per-agent file | transition | profile | workers | built-in). The prefix is optional and always the same content regardless of role.

The new model replaces this with three explicitly named, ordered layers: (1) `apm_core::instructions::generate()` output (from T1/4bee5771, scoped to the role), (2) the project context file at `config.agents.project` (default path `.apm/agents/default/apm.project.md`), (3) the existing role-file cascade unchanged. All three are joined with a blank line between each present layer. The `[agents]` config key changes from `instructions` to `project`; the old key is deprecated — if present without `project`, use it as layer 2 and emit a deprecation warning.

`explain_system_prompt` and `format_provenance` must be updated so `apm prompt --explain` shows the source for all three layers rather than a separate "prefix" line plus a single "system prompt" line.

### Acceptance criteria

- [ ] `build_system_prompt` output, when all three layers are present, contains Layer 1 text, then a blank line, then Layer 2 text, then a blank line, then Layer 3 text — in that order
- [ ] When `agents.project` is not configured (None or empty string), Layer 2 is absent and the output is Layer 1 + blank line + Layer 3 with no extra blank line or gap
- [ ] When `agents.project` names a file that cannot be read, `build_system_prompt` returns an error whose message contains `"agents.project"` and the configured path
- [ ] `AgentsConfig` deserialises `project = "..."` from the `[agents]` section of `config.toml` and stores it as `project: Option<PathBuf>`
- [ ] When `[agents].instructions` is set and `[agents].project` is absent, `build_system_prompt` uses the `instructions` path as Layer 2 and emits a deprecation warning to stderr
- [ ] `apm prompt --explain` output labels all three layers: a `layer 1:` line for apm instructions (dynamic), a `layer 2:` line for the project file path (or "not configured"), and a `layer 3:` line for the cascade winner
- [ ] `apm prompt --agent A --role R` output begins with the content returned by `instructions::generate(root, Some(R), &[])`

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
| 2026-05-22T23:23Z | — | new | philippepascal |
| 2026-05-22T23:50Z | new | groomed | philippepascal |
| 2026-05-23T00:20Z | groomed | in_design | philippepascal |