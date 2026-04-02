+++
id = "e2e3d958"
title = "apm-server: /api/me endpoint, OTP generation, session store, and localhost bypass"
state = "new"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/e2e3d958-apm-server-api-me-endpoint-otp-generatio"
created_at = "2026-04-02T20:54:13.959036Z"
updated_at = "2026-04-02T20:54:13.959036Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["4cec7a17", "90ebf40b"]
+++

## Spec

### Problem

apm-server has no authentication and no way to identify the current user. Any client that can reach the server gets full access. External clients (phone, remote laptop) need a secure auth scheme, and the UI needs a `/api/me` endpoint to know the current user for default filtering. The auth foundation (OTP generation, session store, localhost bypass, session cookie) must be in place before registration and login ceremonies can be built. See `initial_specs/DESIGN-users.md` point 5.

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
| 2026-04-02T20:54Z | — | new | apm |