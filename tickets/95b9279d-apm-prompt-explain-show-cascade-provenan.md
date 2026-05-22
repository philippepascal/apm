+++
id = "95b9279d"
title = "apm prompt --explain: show cascade provenance instead of prompt text"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/95b9279d-apm-prompt-explain-show-cascade-provenan"
created_at = "2026-05-22T10:22:16.387302Z"
updated_at = "2026-05-22T10:23:50.807696Z"
+++

## Spec

### Problem

`build_system_prompt()` in `apm-core/src/start.rs` resolves the agent system prompt through a 5-level cascade (level 0: per-agent file, 1: transition.instructions, 2: profile.instructions, 3: workers.instructions, 4: built-in default) preceded by an `agents.instructions` prefix layer. When a spawned worker behaves unexpectedly — wrong persona, wrong instructions — there is no way to know which level won or which file was read without manually grepping config files and checking the filesystem. `apm prompt <id>` currently prints the full assembled prompt, which does not tell the user *why* that content was chosen.

`apm prompt <id> --explain` should print a compact provenance table that names the source of each layer — which file or config path supplied the prefix, which cascade level won and what its source is, and which levels were checked or configured but not used. The flag makes the debugging loop fast: a supervisor can confirm at a glance that the right file is winning without reading a full prompt dump.

### Acceptance criteria

- [ ] `apm prompt <id> --explain` prints a provenance table to stdout instead of the prompt text
- [ ] The `prefix:` line names the `agents.instructions` file path when configured, or `none` when not configured
- [ ] The `system prompt:` line names the cascade level number (0–4), its fixed label, and its source (file path or `built-in default`)
- [ ] All cascade levels that did not win appear under `skipped:` with their fixed label and their reason (`none set`, `file absent: <path>`, or `not reached`)
- [ ] `--agent` and `--role` override flags work together with `--explain` and are reflected in the provenance output
- [ ] `apm prompt --explain` (no ticket ID) exits non-zero with a message indicating that `--explain` requires a ticket ID
- [ ] Unit tests cover: level 0 wins (per-agent file present), level 4 wins (built-in default), and prefix layer configured

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
| 2026-05-22T10:22Z | — | new | philippepascal |
| 2026-05-22T10:23Z | new | groomed | philippepascal |
| 2026-05-22T10:23Z | groomed | in_design | philippepascal |