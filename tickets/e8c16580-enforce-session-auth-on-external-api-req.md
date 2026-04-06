+++
id = "e8c16580"
title = "Enforce session auth on external API requests"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e8c16580-enforce-session-auth-on-external-api-req"
created_at = "2026-04-06T17:41:57.255655Z"
updated_at = "2026-04-06T17:49:26.429541Z"
+++

## Spec

### Problem

The apm-server has a complete WebAuthn/passkey registration and login flow that issues session cookies (__Host-apm-session), but no API route actually validates the session. All 20+ API endpoints (ticket CRUD, sync, work management, batch operations) accept requests from any client without authentication. This means anyone who can reach the server over the network can read and mutate all project data. The auth system needs a middleware layer that: (1) allows localhost requests through without a session (preserving CLI/agent access), and (2) requires a valid session cookie for all external requests, returning 401 otherwise.

### Acceptance criteria

- [ ] GET `/api/tickets` from an external IP with no session cookie returns 401 with `{"error":"unauthorized"}`
- [ ] GET `/api/tickets` from an external IP with a valid `__Host-apm-session` cookie returns 200
- [ ] GET `/api/tickets` from an external IP with an expired or invalid session token returns 401
- [ ] GET `/api/tickets` from loopback (127.0.0.1 or ::1) with no session cookie returns 200
- [ ] POST `/api/auth/login/challenge` from an external IP with no session cookie returns 200 (auth routes stay open)
- [ ] POST `/api/auth/register/challenge` from an external IP with no session cookie returns 200
- [ ] GET `/health` from an external IP with no session cookie returns 200
- [ ] All protected routes (`/api/sync`, `/api/clean`, `/api/tickets/:id`, `/api/tickets/:id/body`, `/api/tickets/:id/transition`, `/api/tickets/batch/*`, `/api/queue`, `/api/workers`, `/api/workers/:pid`, `/api/work/*`, `/api/agents/config`, `/api/log/stream`, `/api/epics`, `/api/epics/:id`, `/api/me`, `/api/auth/otp`, `/api/auth/sessions`) return 401 when called externally without a valid session
- [ ] Existing localhost-only guards in `otp_handler` and session handlers continue to work after the middleware is added

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
| 2026-04-06T17:41Z | — | new | philippepascal |
| 2026-04-06T17:43Z | new | groomed | apm |
| 2026-04-06T17:49Z | groomed | in_design | philippepascal |