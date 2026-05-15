+++
id = "6790d8c9"
title = "Decide and act on dead config.agents.instructions field"
state = "in_design"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6790d8c9-decide-and-act-on-dead-config-agents-ins"
created_at = "2026-05-14T21:14:56.708735Z"
updated_at = "2026-05-15T21:49:29.781217Z"
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

- [ ] When `agents.instructions` is set in `.apm/config.toml` and the referenced file exists, its content is prepended to every prompt assembled by `build_system_prompt()`, separated from the cascade-resolved body by a blank line\n- [ ] When `agents.instructions` is unset (the default), `build_system_prompt()` output is identical to what ba121f45 produces — no regression\n- [ ] When `agents.instructions` is set to an empty string (`instructions = ""`), `build_system_prompt()` treats it as unset — no prefix is injected and no error is raised\n- [ ] When `agents.instructions` is set but the file is missing, `build_system_prompt()` returns a hard error: `agents.instructions: file not found: <path>`\n- [ ] `apm prompt <id>` output includes the `agents.instructions` prefix when the field is configured\n- [ ] All three call sites in `start.rs` (`run`, `run_next`, `spawn_next_worker`) and the new `prompt::run()` from ba121f45 pass the agents instructions path to `build_system_prompt()`

### Out of scope

- The undocumented `apm agents` command described in `docs/commands.md` (separate unimplemented feature)
- Changing the cascade priority order within `build_system_prompt()` — that is ba121f45 scope
- Per-agent or per-role filtering of the `agents.instructions` content
- Any changes to `workers.instructions` behaviour

### Approach

Decision: **Option A — wire the field in**.

`WorkersConfig::instructions` (cascade Level 3 after ba121f45) replaces the entire resolved prompt. `agents.instructions`, by contrast, is intended as a prefix injected on top of whatever the cascade produces — a project-wide context layer that every worker receives regardless of which cascade level won. The project itself already sets `instructions = ".apm/agents/default/agents.md"` in `.apm/config.toml`, confirming real intent. Removing the field (Option B) would silently discard that config entry and leave the agents.md content unreachable by small-model workers.

**`apm-core/src/start.rs`** — Add parameter `agents_instructions: Option<&Path>` to `build_system_prompt()` (renamed from `resolve_system_prompt()` in ba121f45). Resolve the effective path as follows: if the option is `None`, or if the path stringifies to an empty string, skip the prefix entirely. Otherwise read the file with `std::fs::read_to_string(root.join(path))`; on missing file bail with `"agents.instructions: file not found: {path}"`; on success trim trailing whitespace from the file content then return `format!("{prefix}\n\n{base}")`. This guarantees exactly one blank line between prefix and body regardless of how many trailing newlines the file contains. Update the three call sites (`run` ~line 362, `run_next` ~line 566, `spawn_next_worker` ~line 770) to pass `config.agents.instructions.as_deref()`.

**`apm-core/src/prompt.rs`** (new from ba121f45) — The `prompt::run()` function also calls `build_system_prompt()`; pass `config.agents.instructions.as_deref()` there too.

**No other changes** — `AgentsConfig.instructions: Option<PathBuf>` in `config.rs` stays as-is; `workers.instructions` behaviour is unchanged. Implement on top of the ba121f45 branch since this ticket modifies `build_system_prompt()` introduced there.

### Open questions


### Amendment requests

- [x] Specify behaviour when agents.instructions is configured as an empty string (e.g. instructions = "" in config.toml). Either treat as unset (no prefix injected) — the natural read of an empty path — or bail with the file-not-found error. Add an AC for whichever you pick so an implementer doesn't have to guess.
- [x] Specify the trailing-newline behaviour of the prefix. Approach says 'format!("{prefix}\n\n{base}")' — if the prefix file ends with its own newline, that yields three consecutive newlines. Trim trailing whitespace from the prefix before formatting, or document that the convention is 'one or more separators is fine'.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-14T21:14Z | — | new | philippe|philippepascal |
| 2026-05-14T21:22Z | new | groomed | philippe |
| 2026-05-15T01:53Z | groomed | in_design | philippe |
| 2026-05-15T02:01Z | in_design | specd | default-0515-0153-fbd8 |
| 2026-05-15T19:56Z | specd | ammend | philippe |
| 2026-05-15T21:48Z | ammend | in_design | philippe |