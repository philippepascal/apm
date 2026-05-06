+++
id = "4726eac0"
title = "Add phi4 and pi-phi4 worker profiles to config"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4726eac0-add-phi4-and-pi-phi4-worker-profiles-to-"
created_at = "2026-05-06T19:06:21.963164Z"
updated_at = "2026-05-06T19:06:21.963164Z"
+++

## Spec

### Problem

Add two worker profiles to .apm/config.toml — one for the phi4-ollama wrapper (ticket 42167022) and one for the pi-phi4 wrapper (ticket 80691f15). Both wrappers must exist before this ticket is implemented. Changes are purely additive: append to config.toml only. Profile for phi4: [worker_profiles.phi4] with agent='phi4' and options.model='phi4'. Profile for pi-phi4: [worker_profiles.pi-phi4] with agent='pi' and options.model='phi4'. No workflow changes — profiles are defined but not wired to any transition yet; the supervisor will do that after testing. Also verify .apm/config.toml does not already have a [worker_profiles] section to avoid duplication.

### Acceptance criteria

Checkboxes; each one independently testable.

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
