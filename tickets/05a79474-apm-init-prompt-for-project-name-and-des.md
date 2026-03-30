+++
id = "05a79474"
title = "apm init: prompt for project name and description"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "25666"
branch = "ticket/05a79474-apm-init-prompt-for-project-name-and-des"
created_at = "2026-03-30T23:56:25.325030Z"
updated_at = "2026-03-30T23:56:56.569437Z"
+++

## Spec

### Problem

apm init generates a [project] section in apm.toml with empty name and description fields. Users must manually edit the file after init to fill these in.

apm init should interactively prompt for project name and description when stdin is a TTY. If the user presses Enter without input, or if stdin is not a TTY (e.g. piped or scripted), the fields are left empty. No input is required — the prompts are optional.

Example generated config with user input:

[project]
name = "apm"
description = "Git-native, agent-first project management tool"

### Acceptance criteria

- [ ] hello

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T23:56Z | — | new | philippepascal |
| 2026-03-30T23:56Z | new | in_design | philippepascal |