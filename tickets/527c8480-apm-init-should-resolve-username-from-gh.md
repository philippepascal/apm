+++
id = "527c8480"
title = "apm init should resolve username from gh when available"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/527c8480-apm-init-should-resolve-username-from-gh"
created_at = "2026-04-24T06:27:54.558050Z"
updated_at = "2026-04-24T06:27:54.558050Z"
+++

## Spec

### Problem

apm init calls prompt_username() in apm/src/cmd/init.rs:27-31 on first run even when gh is authenticated, because has_git_host is only true after .apm/config.toml exists. Expected: when gh auth status succeeds, default the prompt to the output of: gh api user -q .login (Enter to accept or override); fall back to blank-default only when gh is unavailable or unauthenticated.

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
| 2026-04-24T06:27Z | — | new | philippepascal |
