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

When a user runs `apm init` for the first time, the project name is silently derived from the directory name and no description is ever collected. The generated `.apm/config.toml` always contains an empty `description` field. This means every new APM project starts with a generic, directory-derived name and no human-readable description — both of which are useful for `apm list`, reporting, and context given to agents.

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