+++
id = "e2e3d958"
title = "apm-server: /api/me endpoint, OTP generation, session store, and localhost bypass"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "64526"
branch = "ticket/e2e3d958-apm-server-api-me-endpoint-otp-generatio"
created_at = "2026-04-02T20:54:13.959036Z"
updated_at = "2026-04-02T23:45:23.811161Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["4cec7a17", "90ebf40b"]
+++

## Spec

### Problem

apm-server has no authentication and no way to identify the current user beyond the localhost case implemented in ticket #90ebf40b. Any client that can reach the server gets full access. External clients (phone, remote laptop) need a secure auth scheme, but before the WebAuthn registration and login ceremonies can be built, the underlying infrastructure must exist: a place to store short-lived OTPs, a place to store authenticated sessions, a mechanism for localhost requests to bypass auth entirely, and a `/api/me` endpoint that answers correctly for all three request categories (localhost, authenticated session, unauthenticated remote).

Ticket #90ebf40b already implements the localhost case of `/api/me` (reading `.apm/local.toml`). This ticket extends that endpoint to also handle session-authenticated remote requests, and adds the OTP generation endpoint (`POST /api/auth/otp`) that `apm register` will call (CLI command is a separate ticket). It does not implement the WebAuthn ceremonies or any session issuance — those depend on this foundation.

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
| 2026-04-02T23:22Z | new | groomed | apm |
| 2026-04-02T23:45Z | groomed | in_design | philippepascal |