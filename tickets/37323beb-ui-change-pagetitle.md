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

- [ ] The browser tab title reads `apm: <reponame>-<username>` (e.g. `apm: apm-philippepascal`) once the page has loaded and the API response is available
- [ ] `<reponame>` matches the `name` field under `[project]` in `.apm/config.toml`
- [ ] `<username>` matches the value returned by `/api/me` for the current session
- [ ] The static `<title>` fallback in `index.html` is updated to `apm` (shown briefly before JS hydrates)
- [ ] If the `/api/me` fetch fails or is pending, the title falls back to the static `index.html` value (`apm`) rather than showing a broken or empty string

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