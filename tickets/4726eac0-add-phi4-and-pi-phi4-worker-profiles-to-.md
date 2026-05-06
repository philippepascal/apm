+++
id = "4726eac0"
title = "Add phi4 and pi-phi4 worker profiles to config"
state = "specd"
priority = 0
effort = 1
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4726eac0-add-phi4-and-pi-phi4-worker-profiles-to-"
created_at = "2026-05-06T19:06:21.963164Z"
updated_at = "2026-05-06T21:36:16.168926Z"
depends_on = ["42167022", "80691f15"]
+++

## Spec

### Problem

`.apm/config.toml` currently has no `[worker_profiles]` section. Tickets 42167022 and 80691f15 introduce two new agent wrappers — `phi4` (a direct Ollama wrapper) and `pi` (a pi CLI wrapper) — but without corresponding profile entries in config, APM cannot resolve them when a transition specifies `profile = "phi4"` or `profile = "pi-phi4"`. The supervisor needs the profiles registered before wiring them to any workflow transition.

The required change is purely additive: append two named profiles to `config.toml`. No existing fields are modified. The profiles are not wired to any transition in this ticket; that step follows after the supervisor validates the agents end-to-end.

### Acceptance criteria

- [ ] `.apm/config.toml` contains a `[worker_profiles.phi4]` section with `agent = "phi4"`
- [ ] `.apm/config.toml` contains a `[worker_profiles.phi4.options]` section with `model = "phi4"`
- [ ] `.apm/config.toml` contains a `[worker_profiles.pi-phi4]` section with `agent = "pi"`
- [ ] `.apm/config.toml` contains a `[worker_profiles.pi-phi4.options]` section with `model = "phi4"`
- [ ] No existing section in `.apm/config.toml` is modified or removed
- [ ] `.apm/config.toml` contains exactly one occurrence of the string `[worker_profiles` (no duplicate section headers)

### Out of scope

- Wiring either profile to a workflow transition (`profile =` in `workflow.toml`) — the supervisor does that after testing
- Creating or modifying the wrapper files themselves (covered by tickets 42167022 and 80691f15)
- Adding `env`, `role`, `instructions`, or `container` fields to either profile
- End-to-end validation that the agents produce correct output

### Approach

The only file that changes is `.apm/config.toml`. The change is a pure append — no existing lines are touched.

1. Confirm no `[worker_profiles` header exists yet:
   ```sh
   grep -c '\[worker_profiles' .apm/config.toml
   ```
   If the count is non-zero, stop and report — do not append.

2. Append the following block to the end of `.apm/config.toml`:
   ```toml

   [worker_profiles.phi4]
   agent = "phi4"

   [worker_profiles.phi4.options]
   model = "phi4"

   [worker_profiles.pi-phi4]
   agent = "pi"

   [worker_profiles.pi-phi4.options]
   model = "phi4"
   ```

3. Commit:
   ```
   feat(config): add phi4 and pi-phi4 worker profiles
   ```
   Stage only `.apm/config.toml`.

The `agent` values (`"phi4"` and `"pi"`) must match the directory names under `.apm/agents/` that tickets 42167022 and 80691f15 create. APM resolves `agent = "phi4"` to `.apm/agents/phi4/wrapper.*` at spawn time, so no further config is needed.

`options.model` is forwarded to the wrapper as the `APM_OPT_MODEL` environment variable, which both wrappers read to select the Ollama model.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-06T19:06Z | — | new | philippepascal |
| 2026-05-06T20:48Z | new | groomed | philippepascal |
| 2026-05-06T21:33Z | groomed | in_design | philippepascal |
| 2026-05-06T21:36Z | in_design | specd | claude-0506-2133-9fd8 |
