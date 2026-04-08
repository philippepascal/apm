+++
id = "6f3f7790"
title = "Update README and help text with ownership model"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/6f3f7790-update-readme-and-help-text-with-ownersh"
created_at = "2026-04-08T15:32:38.451292Z"
updated_at = "2026-04-08T15:32:38.451292Z"
epic = "18dab82d"
target_branch = "epic/18dab82d-ticket-ownership-model"
depends_on = ["751f65f6", "b52fc7f4"]
+++

## Spec

### Problem

The README and CLI help text do not document the ownership model: who owns tickets, how dispatchers filter by owner, how to assign/reassign, the two identity modes (config vs GitHub). Users have no way to understand the ownership workflow without reading code.

### Acceptance criteria

- [ ] README has a section explaining ticket ownership (author vs owner, who can reassign, dispatcher behavior)
- [ ] README documents `apm assign` and `apm epic set <id> owner`
- [ ] README documents identity setup (local.toml username for config mode, git_host for GitHub mode)
- [ ] `apm assign --help` text is clear and accurate
- [ ] `docs/commands.md` updated with ownership-related commands
- [ ] Happy path walkthrough reflects ownership (supervisor creates and owns, dispatches to workers)

### Out of scope

Documenting the ownership spec itself (already in docs/ownership-spec.md). API documentation for apm-server.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:32Z | — | new | philippepascal |