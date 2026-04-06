+++
id = "e8c16580"
title = "Enforce session auth on external API requests"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/e8c16580-enforce-session-auth-on-external-api-req"
created_at = "2026-04-06T17:41:57.255655Z"
updated_at = "2026-04-06T17:43:43.817351Z"
+++

## Spec

### Problem

The apm-server has a complete WebAuthn/passkey registration and login flow that issues session cookies (__Host-apm-session), but no API route actually validates the session. All 20+ API endpoints (ticket CRUD, sync, work management, batch operations) accept requests from any client without authentication. This means anyone who can reach the server over the network can read and mutate all project data. The auth system needs a middleware layer that: (1) allows localhost requests through without a session (preserving CLI/agent access), and (2) requires a valid session cookie for all external requests, returning 401 otherwise.

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
| 2026-04-06T17:41Z | — | new | philippepascal |
| 2026-04-06T17:43Z | new | groomed | apm |
