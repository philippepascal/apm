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

- [ ] `apm prompt` with no arguments exits 0 and prints an `Agents:` line whose value is the sorted, comma-space-separated list of subdirectory names under `.apm/agents/`
- [ ] `apm prompt` with no arguments exits 0 and prints a `Roles:` line whose value is the sorted, comma-space-separated list of unique role names extracted from `apm.<role>.md` filenames across all agent directories
- [ ] The two output lines align their values at the same column (labels padded to equal width)
- [ ] `apm prompt --agent <name>` with no ID triggers discovery mode and produces the same output as bare `apm prompt`
- [ ] `apm prompt --role <name>` with no ID triggers discovery mode and produces the same output as bare `apm prompt`
- [ ] When `.apm/agents/` does not exist, discovery exits 0 and prints `Agents:` and `Roles:` lines with empty values rather than erroring
- [ ] `apm prompt <id>` with a valid ticket ID behaves identically to the pre-change implementation
- [ ] `apm prompt <id> --agent <a> --role <r>` continues to work as before

### Out of scope

- Changing how `apm start` or any other command resolves agents or roles
- Filtering the discovery output by the supplied `--agent` or `--role` flag value
- Discovery for commands other than `apm prompt`
- Validating that discovered agent/role pairs have usable instructions (that is a concern for the prompt-building path, not discovery)

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