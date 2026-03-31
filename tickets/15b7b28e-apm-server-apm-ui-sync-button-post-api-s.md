+++
id = "15b7b28e"
title = "apm-server + apm-ui: sync button (POST /api/sync)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "5398"
branch = "ticket/15b7b28e-apm-server-apm-ui-sync-button-post-api-s"
created_at = "2026-03-31T06:13:15.004948Z"
updated_at = "2026-03-31T07:10:36.832701Z"
+++

## Spec

### Problem

The UI has no way to pull the latest ticket state from git branches. Add POST /api/sync that runs apm sync logic and refreshes all ticket data. A sync button in the UI (with keyboard shortcut) triggers this and shows a loading state while in progress. Full spec context: initial_specs/UIdraft_spec_starter.md Step 13. Requires Step 4.

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
| 2026-03-31T06:13Z | — | new | apm |
| 2026-03-31T07:10Z | new | in_design | philippepascal |