+++
id = "610be42e"
title = "apm-core: write author from identity on ticket creation, remove agent field"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "69160"
branch = "ticket/610be42e-apm-core-write-author-from-identity-on-t"
created_at = "2026-04-02T20:53:55.085303Z"
updated_at = "2026-04-02T23:30:06.435781Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["4cec7a17"]
+++

## Spec

### Problem

New tickets set `author` from the `APM_AGENT_NAME` environment variable (or fall back to `"apm"`), conflating the ephemeral worker name with a permanent creator identity. Meanwhile, the `agent` frontmatter field tracks the current worker name — but workers are single-use, resumability does not depend on it, and tying frontmatter to a specific naming convention is the wrong direction (DESIGN-users.md point 2).

There is no mechanism today to resolve a real human username for `author`. The design calls for reading `.apm/local.toml` (a gitignored, per-machine file) for the `username` key, falling back to `"apm"` when absent (DESIGN-users.md points 1 and 3).

This ticket adds the identity-resolution function in `apm-core`, wires it into `apm new`, and removes the `agent` field from frontmatter writes and from `apm list`/`apm show` output.

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
| 2026-04-02T23:22Z | new | groomed | apm |
| 2026-04-02T23:30Z | groomed | in_design | philippepascal |