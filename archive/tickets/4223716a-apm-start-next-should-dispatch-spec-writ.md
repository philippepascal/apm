+++
id = "4223716a"
title = "apm start --next should dispatch spec-writer agent for new/ammend tickets"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
agent = "66030"
branch = "ticket/4223716a-apm-start-next-should-dispatch-spec-writ"
created_at = "2026-03-30T20:52:32.229319Z"
updated_at = "2026-03-30T22:51:52.674650Z"
+++

## Spec

### Problem

`apm start` always dispatches a worker using `.apm/worker.md` as the system prompt, regardless of the ticket's state. This is hardcoded in `apm-core/src/start.rs` and ignores the `instructions` field already present on each state in `apm.toml`.

The config already routes correctly:
- `new`, `ammend`, `in_design` states have `instructions = "apm.spec-writer.md"`
- `ready`, `in_progress` states have `instructions = "apm.worker.md"`

The actual files live at `.apm/apm.spec-writer.md` and `.apm/apm.worker.md` (with the `apm.` prefix). The `instructions` value in config is a filename resolved relative to `.apm/`.

The fix: in `start.rs`, after resolving the ticket's current state, look up that state's `instructions` field from config and load `.apm/<instructions>` as the system prompt instead of the hardcoded `.apm/worker.md`.

### Acceptance criteria

- [ ] `apm start <id> --spawn` on a ticket in `new` state spawns a subprocess using `.apm/apm.spec-writer.md` as the system prompt
- [ ] `apm start <id> --spawn` on a ticket in `ammend` state spawns a subprocess using `.apm/apm.spec-writer.md` as the system prompt
- [ ] `apm start --next --spawn` dispatches a spec-writer subprocess when the next ticket is in `new` state
- [ ] `apm start --next --spawn` dispatches a spec-writer subprocess when the next ticket is in `ammend` state
- [ ] `apm start --next --spawn` dispatches a worker subprocess (using `apm.worker.md`) when the next ticket is in `ready` state
- [ ] When `.apm/apm.spec-writer.md` is absent, all spec-writer dispatch paths fall back to `.apm/apm.worker.md`
- [ ] The initial user message to a spec-writer subprocess begins with "You are a Spec-Writer agent assigned to ticket #<id>."
- [ ] The initial user message to a worker subprocess continues to begin with "You are a Worker agent assigned to ticket #<id>."

### Out of scope

- Adding a spec-writer role section to `.apm/agents.md` (agents.md is a user-editable file; this ticket touches only `start.rs`)
- Changing the default `apm init` config to add `instructions` fields on states
- Container (`docker`) spawn path — same logic applies but is not separately tested here

### Approach

Three spawn sites in `apm-core/src/start.rs` all need the same fix.

**Helper function** — extract a `resolve_system_prompt` function (or inline the same logic in each site) that selects the prompt based on the ticket's state before the `command:start` transition fires:

```rust
fn resolve_system_prompt(root: &Path, pre_transition_state: &str) -> String {
    let spec_writer_states = ["new", "ammend"];
    if spec_writer_states.contains(&pre_transition_state) {
        let p = root.join(".apm/apm.spec-writer.md");
        if let Ok(content) = std::fs::read_to_string(&p) {
            return content;
        }
    }
    let p = root.join(".apm/apm.worker.md");
    std::fs::read_to_string(p)
        .unwrap_or_else(|_| "You are an APM worker agent.".to_string())
}
```

**User message prefix** — add a parallel helper to produce the role prefix:

```rust
fn agent_role_prefix(pre_transition_state: &str, id: &str) -> String {
    let spec_writer_states = ["new", "ammend"];
    if spec_writer_states.contains(&pre_transition_state) {
        format!("You are a Spec-Writer agent assigned to ticket #{id}.")
    } else {
        format!("You are a Worker agent assigned to ticket #{id}.")
    }
}
```

**Sites to update** (all in `apm-core/src/start.rs`):

1. `run()` — lines ~233–240: replace the hardcoded `apm.worker.md` load with `resolve_system_prompt(root, &old_state)`, and replace the `"You are a Worker agent..."` prefix with `agent_role_prefix(&old_state, &id)`.

2. `run_next()` — lines ~380–393: replace the `if !prompt.is_empty() { prompt } else { apm.worker.md }` block with `resolve_system_prompt(root, &old_state)`, and update the `ticket_content` format string to use `agent_role_prefix`.

3. `spawn_next_worker()` — lines ~535–549: same replacement as `run_next()`.

**Note on existing `instructions_text` logic in `run_next()` / `spawn_next_worker()`:** The state `instructions` config field (read into `instructions_text`) is a more general mechanism. After this change, the hardcoded spec-writer selection takes precedence for `new`/`ammend`, so the `instructions_text` path is still useful for other custom states but no longer needed for these two. Leave it in place — do not remove it.

**Order of steps:**
1. Add `resolve_system_prompt` and `agent_role_prefix` as private functions near the bottom of `start.rs`.
2. Update `run()` spawn block.
3. Update `run_next()` spawn block.
4. Update `spawn_next_worker()` spawn block.
5. Add integration tests covering spec-writer selection and worker selection.

### Open questions



### Amendment requests

- [ ] The Approach hardcodes `["new", "ammend"]` as the spec-writer states, but the config already encodes this via the `instructions` field on each `[[workflow.states]]` entry. Replace the hardcoded array with a config lookup: find the state matching `pre_transition_state` in `config.workflow.states`, read its `instructions` field, and load `.apm/<instructions>` as the system prompt. This way adding a new spec-writing state only requires a config change, not a code change.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T20:52Z | — | new | philippepascal |
| 2026-03-30T20:52Z | new | in_design | philippepascal |
| 2026-03-30T21:00Z | in_design | specd | claude-0330-2100-b7f2 |
| 2026-03-30T21:04Z | specd | ammend | apm |
| 2026-03-30T21:05Z | ammend | in_design | philippepascal |
| 2026-03-30T21:09Z | in_design | specd | claude-0330-2145-f8c2 |
| 2026-03-30T21:10Z | specd | ammend | philippepascal |
| 2026-03-30T21:10Z | ammend | in_design | philippepascal |
| 2026-03-30T21:22Z | in_design | specd | claude-0330-2200-w9k4 |
| 2026-03-30T22:51Z | specd | closed | apm |