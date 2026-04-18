+++
id = "37323beb"
title = "UI change pagetitle"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/37323beb-ui-change-pagetitle"
created_at = "2026-04-18T01:16:01.610380Z"
updated_at = "2026-04-18T01:16:40.590679Z"
+++

## Spec

### Problem

The browser tab title for the APM UI is hardcoded to `apm-ui` in `apm-ui/index.html`. This static value gives users no contextual information about which project or account they are working in — a usability issue when multiple APM instances are open in the same browser.

The title should instead read `apm: <reponame>-<username>`, e.g. `apm: apm-philippepascal`, so the tab immediately identifies both the project and the logged-in user.

The project name is available from `[project] name` in `.apm/config.toml` (loaded via `apm_core::config::Config::load`); the username is already returned by the `/api/me` endpoint. Neither value is currently surfaced to the frontend.

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
| 2026-04-18T01:16Z | — | new | philippepascal |
| 2026-04-18T01:16Z | new | groomed | apm |
| 2026-04-18T01:16Z | groomed | in_design | philippepascal |