+++
id = "8c7d47f0"
title = "apm-server + apm-ui: state transition API and buttons"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "82538"
branch = "ticket/8c7d47f0-apm-server-apm-ui-state-transition-api-a"
created_at = "2026-03-31T06:12:47.638355Z"
updated_at = "2026-03-31T06:42:17.687543Z"
+++

## Spec

### Problem

The ticket detail panel (added in Step 6) is read-only: a supervisor looking at a ticket in the UI cannot change its state without switching to the CLI. This blocks the supervisor from completing their core workflow — reviewing specs, approving tickets, sending amendments — entirely from the browser.

**Current state (after Step 6):** The right column renders full ticket markdown and updates reactively when a ticket is selected. State is shown as a badge but there are no controls to change it.

**Desired state:**
- A new `POST /api/tickets/:id/transition` endpoint accepts `{"to":"<state>"}` and executes the apm-core state machine, including all guards (spec validation, criteria checks, valid-transition enforcement).
- The ticket detail panel grows a row of action buttons — one per valid transition from the current state — derived from the workflow config. Each button label comes from the `label` field in the transition config (or `→ {to}` as fallback).
- A "Keep at {state}" button is always present as a no-op affordance, matching the CLI `apm review` menu.
- Transition errors (invalid transition, precondition failure) surface inline near the buttons.
- After a successful transition the panel refreshes with the new state and new available transitions automatically.

**Who is affected:** Supervisors using the web UI to review and progress tickets.

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
| 2026-03-31T06:12Z | — | new | apm |
| 2026-03-31T06:42Z | new | in_design | philippepascal |