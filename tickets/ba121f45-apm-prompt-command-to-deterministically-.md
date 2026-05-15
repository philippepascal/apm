+++
id = "ba121f45"
title = "apm prompt command to deterministically build worker system prompt"
state = "in_design"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ba121f45-apm-prompt-command-to-deterministically-"
created_at = "2026-05-14T21:14:16.356953Z"
updated_at = "2026-05-15T01:33:41.389488Z"
+++

## Spec

### Problem

Workers spawned via `apm start`, `apm work`, and the UI dispatch loop all call `resolve_system_prompt()` in `apm-core/src/start.rs`, which applies a 5-level priority cascade to produce the system prompt. The cascade is: (0) `transition.instructions`, (1) profile.instructions, (2) workers.instructions, (3) `.apm/agents/<agent>/apm.<role>.md`, (4) built-in default. Because Level 0 always wins when a transition has `instructions` set — and nearly every transition in `workflow.toml` does — the per-agent files at Level 3 are unreachable dead code. A `pi`-agent transition still gets the `default` spec-writer prompt even if `.apm/agents/pi/apm.spec-writer.md` exists. There is also no CLI surface to inspect what any given (agent, role, ticket) tuple would receive; the only way to see the real prompt is to launch a live worker. Add an `apm prompt <ticket-id>` command that deterministically assembles and prints the system prompt for a ticket's current state without spawning a worker. Promote the per-agent file lookup to the highest-priority level so agent-specific overrides actually take effect. Consolidate the three spawn paths — `run()`, `run_next()`, and `spawn_next_worker()` — onto a single `build_system_prompt()` function so the CLI output is guaranteed identical to what a worker receives.

### Acceptance criteria

- [ ] `apm prompt <ticket-id>` prints to stdout the system prompt that would be used if the ticket's current transition fired, and exits 0
- [ ] `apm prompt <ticket-id> --agent <name>` overrides the resolved agent for the inspection without affecting the ticket
- [ ] `apm prompt <ticket-id> --role <name>` overrides the resolved role for the inspection without affecting the ticket
- [ ] When `.apm/agents/<agent>/apm.<role>.md` exists it is used in preference to `transition.instructions`, profile.instructions, and workers.instructions
- [ ] When no per-agent file exists, `transition.instructions` is used as before (backward compatible with existing `workflow.toml` transitions)
- [ ] All three spawn paths (`run()`, `run_next()`, `spawn_next_worker()`) produce the same prompt as `apm prompt` for the same (agent, role, ticket) inputs
- [ ] `apm prompt` exits non-zero with a clear message when no instructions can be resolved for the given tuple
- [ ] `apm prompt` does not spawn a worker, modify any ticket, or write any temp files

### Out of scope

- Changes to how tickets are selected or dispatched (priority, epic filtering, etc.)
- Changes to the user message portion of the prompt (role prefix, ticket content, epic bundle, dependency bundle)
- New instruction file formats or templating beyond what already exists
- Per-ticket instruction overrides (e.g. a field on the ticket frontmatter)
- Changes to `apm agent` or any other subcommand not directly involved in prompt resolution
- Validation that instruction file contents are well-formed

### Approach

**Step 1 — Rename and restructure `resolve_system_prompt()` in `apm-core/src/start.rs`**\n\n- Rename to `build_system_prompt()`\n- Move the per-agent file check to Level 0 (highest priority, soft: no error if absent): `.apm/agents/<agent>/apm.<role>.md`\n- Level 1 (was Level 0): transition.instructions — hard error if set but missing\n- Level 2 (was Level 1): profile.instructions — hard error if set but missing\n- Level 3 (was Level 2): workers.instructions — hard error if set but missing\n- Level 4 (was Level 3/4): built-in default via `resolve_builtin_instructions(agent, role)`\n- Level 5: hard error — no instructions found\n- All existing error messages for Levels 1–5 remain unchanged\n\n**Step 2 — Add `apm prompt` subcommand**\n\n- Add `Prompt { id: String, agent: Option<String>, role: Option<String> }` to the CLI enum in `apm/src/main.rs`\n- Wire to a new `prompt::run()` in `apm-core/src/prompt.rs` (or a public function in `start.rs`)\n- Implementation: (1) load the ticket by id; (2) find the applicable transition for the ticket's current state using the same lookup as `run()`; (3) resolve agent and role via the normal cascade, then apply CLI overrides; (4) call `build_system_prompt()`; (5) print result to stdout; (6) exit non-zero on error\n\n**Step 3 — Update the three spawn paths**\n\n- In `run()` (line 362), `run_next()` (line 566), and `spawn_next_worker()` (line 770): replace `resolve_system_prompt(...)` with `build_system_prompt(...)`\n- No other changes to those call sites\n\n**Step 4 — No config or schema changes**\n\n- `.apm/workflow.toml` and `.apm/config.toml` formats unchanged; no migration needed\n- Transitions that set `instructions = ...` continue to work unchanged as Level 1 when no per-agent file is present

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-14T21:14Z | — | new | philippe|philippepascal |
| 2026-05-14T21:22Z | new | groomed | philippe |
| 2026-05-14T21:22Z | groomed | in_design | philippe |
| 2026-05-15T01:21Z | in_design | groomed | philippe |
| 2026-05-15T01:29Z | groomed | in_design | philippe |