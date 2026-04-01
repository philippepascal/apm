+++
id = "a595add3"
title = "Split config.toml into focused files for state machine, ticket structure, and project settings"
state = "in_design"
priority = 9
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "42283"
branch = "ticket/a595add3-split-config-toml-into-focused-files-for"
created_at = "2026-04-01T22:27:35.511052Z"
updated_at = "2026-04-01T22:35:07.194666Z"
+++

## Spec

### Problem

The single `.apm/config.toml` file mixes unrelated concerns: project identity, repo and provider settings, sync behaviour, worktree paths, workflow state machine, and ticket section definitions. As the project grows, this becomes hard to navigate and makes it impossible to swap out individual concerns (e.g. use a different state machine) without touching everything else.

The goal is to split it into focused files, likely:
- One for the workflow state machine (the `[[workflow.states]]` blocks — the bulk of the current file)
- One for ticket structure (`[[ticket.sections]]` — defines the body sections spec-writers fill in)
- One for project and infrastructure settings (project name, repos, provider, sync, worktrees)

The spec writer should read `.apm/config.toml` in full to understand what exists today, then propose the right file split and how the loader in `apm-core/src/config.rs` should be changed to load and merge the files.

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T22:27Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:28Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-01T22:35Z | groomed | in_design | philippepascal |
