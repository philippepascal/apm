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

- Updating the title dynamically on route changes (single page; one title is sufficient)
- Showing the full git host repo slug (e.g. `philippepascal/apm`) — the short project name from `[project] name` is used
- Any change to the favicon or other browser-tab metadata
- Multi-tenant / remote-auth scenarios where `repo_name` might differ per session

### Approach

**1. Extend `/api/me` to include `repo_name` — `apm-server/src/main.rs`**

In `me_handler`, after resolving `username`, load the config via `apm_core::config::Config::load(root)` (same pattern as `collaborators_handler`) and read `config.project.name`. Return it alongside `username`:
```json
{ "username": "philippepascal", "repo_name": "apm" }
```
Handle the case where `git_root()` is `None` or config fails to load by defaulting `repo_name` to an empty string (or omit the field — the UI must tolerate absence).

**2. Set `document.title` dynamically — `apm-ui/src/App.tsx`**

Add a `useEffect` (or a custom `useMeta` hook) that fires after the existing `/api/me` fetch resolves. The query result already flows through React Query in `AssignPicker`; lift the `/api/me` query to `App.tsx` (or a layout-level component) so the title can be set once on mount:

```ts
useEffect(() => {
  if (me?.repo_name && me?.username) {
    document.title = `apm: ${me.repo_name}-${me.username}`;
  }
}, [me]);
```

If `me` is undefined / loading / error, leave `document.title` at its default (the static value from `index.html`).

**3. Update the static fallback — `apm-ui/index.html` line 7**

Change `<title>apm-ui</title>` to `<title>apm</title>`. This is what appears before JS loads and serves as the error/loading fallback.

**Order of changes:** backend first (so the new field is present when testing), then UI effect, then `index.html` cleanup.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-18T01:16Z | — | new | philippepascal |
| 2026-04-18T01:16Z | new | groomed | apm |
| 2026-04-18T01:16Z | groomed | in_design | philippepascal |