+++
id = "6790d8c9"
title = "Decide and act on dead config.agents.instructions field"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6790d8c9-decide-and-act-on-dead-config-agents-ins"
created_at = "2026-05-14T21:14:56.708735Z"
updated_at = "2026-05-15T01:53:18.554746Z"
depends_on = ["ba121f45"]
+++

## Spec

### Problem

Today `config.agents.instructions` is declared in `AgentsConfig` (`apm-core/src/config.rs:478`) but **never read anywhere** — it's effectively dead code. Its declared purpose ("Path to an instructions file injected into every worker prompt") matches exactly what we now need: a way to ensure `agents.md` content reaches small-model workers like pi without relying on them to fetch it.

After `apm prompt` (ticket ba121f45) lands, decide whether this field should be honoured:

Option A — wire it in: `apm prompt` prepends the content of `config.agents.instructions` to every assembled prompt. Lets a project ship a single agents.md that flows to every worker automatically.

Option B — remove the dead field: delete it from `AgentsConfig` to avoid the false-friend trap (looks configurable, isn't).

Acceptance: pick A or B with a paragraph of reasoning in Approach, then implement.

### Acceptance criteria

- [ ] When `agents.instructions` is set in `.apm/config.toml` and the referenced file exists, its content is prepended to every prompt assembled by `build_system_prompt()`, separated from the cascade-resolved body by a blank line
- [ ] When `agents.instructions` is unset (the default), `build_system_prompt()` output is identical to what ba121f45 produces — no regression
- [ ] When `agents.instructions` is set but the file is missing, `build_system_prompt()` returns a hard error: `agents.instructions: file not found: <path>`
- [ ] `apm prompt <id>` output includes the `agents.instructions` prefix when the field is configured
- [ ] All three call sites in `start.rs` (`run`, `run_next`, `spawn_next_worker`) and the new `prompt::run()` from ba121f45 pass the agents instructions path to `build_system_prompt()`

### Out of scope

- The undocumented `apm agents` command described in `docs/commands.md` (separate unimplemented feature)
- Changing the cascade priority order within `build_system_prompt()` — that is ba121f45 scope
- Per-agent or per-role filtering of the `agents.instructions` content
- Any changes to `workers.instructions` behaviour

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-14T21:14Z | — | new | philippe|philippepascal |
| 2026-05-14T21:22Z | new | groomed | philippe |
| 2026-05-15T01:53Z | groomed | in_design | philippe |