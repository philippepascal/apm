+++
id = "4726eac0"
title = "Add phi4 and pi-phi4 worker profiles to config"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4726eac0-add-phi4-and-pi-phi4-worker-profiles-to-"
created_at = "2026-05-06T19:06:21.963164Z"
updated_at = "2026-05-06T21:33:21.603315Z"
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

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-06T19:06Z | — | new | philippepascal |
| 2026-05-06T20:48Z | new | groomed | philippepascal |
| 2026-05-06T21:33Z | groomed | in_design | philippepascal |