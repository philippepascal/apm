+++
id = "79326024"
title = "apm init: username prompt, local.toml, gitignore, and collaborators bootstrap"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "philippepascal"
branch = "ticket/79326024-apm-init-username-prompt-local-toml-giti"
created_at = "2026-04-02T20:53:51.576153Z"
updated_at = "2026-04-02T23:26:11.560729Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["4cec7a17"]
+++

## Spec

### Problem

`apm init` does not prompt for a username or write `.apm/local.toml`. There is no collaborators list seeded at init time, and `.apm/local.toml` is not added to `.gitignore`. See `initial_specs/DESIGN-users.md` points 1 and 7.

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
| 2026-04-02T20:53Z | — | new | apm |
| 2026-04-02T23:19Z | new | groomed | apm |
| 2026-04-02T23:26Z | groomed | in_design | philippepascal |
