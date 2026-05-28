+++
id = "8f73686b"
title = "apm init addes a model sonnet line in config.toml"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/8f73686b-apm-init-addes-a-model-sonnet-line-in-co"
created_at = "2026-05-28T01:54:59.492163Z"
updated_at = "2026-05-28T06:11:39.276663Z"
+++

## Spec

### Problem

apm init create this in config.toml:
[workers]
default = "claude/coder"
model = "sonnet"

default is correct, but model isn't: it's part of the manifest for claude/coder and shouldn't defined at all in config.toml.

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
| 2026-05-28T01:54Z | — | new | philippepascal |
| 2026-05-28T06:09Z | new | groomed | philippepascal |
| 2026-05-28T06:11Z | groomed | in_design | philippepascal |
