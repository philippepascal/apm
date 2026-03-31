+++
id = "3b0019a3"
title = "apm-ui: supervisor swimlanes in middle column"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "34099"
branch = "ticket/3b0019a3-apm-ui-supervisor-swimlanes-in-middle-co"
created_at = "2026-03-31T06:11:59.993473Z"
updated_at = "2026-03-31T06:23:39.579037Z"
+++

## Spec

### Problem

The middle column (SupervisorView) is an empty shell from Step 4. It needs to render tickets grouped by state as vertical swimlanes so a supervisor can see at a glance what needs their attention.

Currently there is no way to see supervisor-actionable tickets in the UI. The supervisor must use the CLI to identify what needs review, approval, or unblocking. The swimlane view gives a columnar overview of every ticket in a state that requires supervisor action, making the workscreen the primary interface for the supervision workflow.

The supervisor-actionable states (from config.toml `actionable = ["supervisor"]`) are: **question**, **specd**, **blocked**, **implemented**, and **accepted**. Swimlanes for states with no tickets must be hidden. Tickets within a swimlane are shown as compact summary cards. Clicking a card updates the global `selectedTicketId` in Zustand, which will drive the right-column detail panel (Step 6).

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
| 2026-03-31T06:11Z | — | new | apm |
| 2026-03-31T06:23Z | new | in_design | philippepascal |