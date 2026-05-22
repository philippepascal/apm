+++
id = "3c4fb21a"
title = "apm prompt: make ID optional; show agent/role discovery when called with no ID"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3c4fb21a-apm-prompt-make-id-optional-show-agent-r"
created_at = "2026-05-22T08:01:03.768635Z"
updated_at = "2026-05-22T08:05:07.484125Z"
+++

## Spec

### Problem

`apm prompt` declares its ID argument as `id: String` in the clap struct, making it a required positional. Running the command bare — or with only `--agent`/`--role` flags but no ID — causes clap to abort with a generic missing-argument error before any application code runs. This is unhelpful when a user wants to know what agents and roles are configured in the project before assembling a full prompt invocation.

The desired behaviour is a discovery mode: when no ID is supplied (regardless of whether `--agent` or `--role` are present), the command scans `.apm/agents/` for agent subdirectory names and extracts role names from `apm.<role>.md` filenames within those directories, then prints a two-line summary and exits 0. When an ID is supplied, behaviour is entirely unchanged.

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
| 2026-05-22T08:01Z | — | new | philippepascal |
| 2026-05-22T08:05Z | new | groomed | philippepascal |
| 2026-05-22T08:05Z | groomed | in_design | philippepascal |