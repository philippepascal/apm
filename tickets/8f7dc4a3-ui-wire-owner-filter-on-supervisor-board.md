+++
id = "8f7dc4a3"
title = "UI: wire owner filter on supervisor board and rename agent filter to owner"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/8f7dc4a3-ui-wire-owner-filter-on-supervisor-board"
created_at = "2026-04-04T06:28:20.587222Z"
updated_at = "2026-04-04T06:59:11.698374Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["2b7c4c97"]
+++

## Spec

### Problem

The supervisor board has a filter dropdown labelled "All agents" in SupervisorView.tsx. It is backed by the `agentFilter` state variable and reads `ticket.agent` to build the option list and apply the filter. However, the `Frontmatter` struct (and therefore the API response) has no `agent` field yet — that field is added by the dependency ticket #42f4b3ba. Until the dependency lands the dropdown is populated from nothing and every filter match fails silently.

Once #42f4b3ba and #2b7c4c97 land, the API will return `agent` on each ticket object. This ticket has two jobs: (1) ensure the UI is wired to that real field so the filter works, and (2) rename all user-visible labels and internal identifiers from "agent" to "owner" to match the terminology used throughout the rest of the product (the broader epic is user-management / ownership, not agent tracking).

The `Ticket` TypeScript interface in `types.ts` already has `agent?: string` which matches what the API will return, so no interface change is needed — only renaming of internal state variables, computed values, and the dropdown label.

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
| 2026-04-04T06:28Z | — | new | apm |
| 2026-04-04T06:35Z | new | groomed | apm |
| 2026-04-04T06:59Z | groomed | in_design | philippepascal |