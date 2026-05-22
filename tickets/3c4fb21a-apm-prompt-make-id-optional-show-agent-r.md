+++
id = "3c4fb21a"
title = "apm prompt: make ID optional; show agent/role discovery when called with no ID"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3c4fb21a-apm-prompt-make-id-optional-show-agent-r"
created_at = "2026-05-22T08:01:03.768635Z"
updated_at = "2026-05-22T08:05:00.333411Z"
+++

## Spec

### Problem

apm prompt currently requires a ticket ID as a positional argument (enforced by clap). Running it bare errors with a missing-argument message, which is unhelpful when a user does not yet know what agents or roles are available in the project.

The fix has two parts: (1) make the ID argument optional in the clap definition, (2) add a discovery mode that fires when no ID is supplied — regardless of whether --agent or --role flags are present. Discovery mode reads .apm/agents/ subdirectory names for the agent list and scans for apm.<role>.md filenames across all agent dirs for the role list, then prints:

  Agents:  claude, default, pi
  Roles:   spec-writer, worker

When an ID is supplied, behaviour is unchanged. Partial flags without an ID (e.g. apm prompt --agent claude) also trigger discovery mode rather than attempting a half-assembled prompt with no ticket context.

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
