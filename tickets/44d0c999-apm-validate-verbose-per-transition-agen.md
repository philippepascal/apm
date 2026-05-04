+++
id = "44d0c999"
title = "apm validate --verbose: per-transition agent resolution audit"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/44d0c999-apm-validate-verbose-per-transition-agen"
created_at = "2026-05-04T17:40:24.657468Z"
updated_at = "2026-05-04T17:43:53.175056Z"
epic = "5acea599"
target_branch = "epic/5acea599-flexible-agent-configuration"
depends_on = ["6803b88b"]
+++

## Spec

### Problem

After ticket 6803b88b lands, `instructions` and `role_prefix` can be set directly on each `command:start` transition in `workflow.toml`. Combined with the existing profile → workers → project-agent-file → built-in fallback chain, a spawn transition now resolves its instructions through up to five levels and its role prefix through three. `apm validate` already checks that referenced files exist and that profile names are valid, but it does not show *which value wins* at each level for a given transition. A project author adding a new spawn transition—or debugging why the wrong instructions file is loading—has no way to confirm the effective agent, instructions file, role prefix, and wrapper without reading source code or running a live spawn.\n\n`apm validate --verbose` closes this gap by appending a per-transition agent resolution audit to the normal validate output.

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
| 2026-05-04T17:40Z | — | new | philippepascal |
| 2026-05-04T17:40Z | new | groomed | philippepascal |
| 2026-05-04T17:43Z | groomed | in_design | philippepascal |