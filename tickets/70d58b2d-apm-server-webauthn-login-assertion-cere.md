+++
id = "70d58b2d"
title = "apm-server: WebAuthn login/assertion ceremony and embedded login page"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/70d58b2d-apm-server-webauthn-login-assertion-cere"
created_at = "2026-04-02T20:54:21.301151Z"
updated_at = "2026-04-02T23:22:55.587603Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["8a08637c"]
+++

## Spec

### Problem

Registered devices have no way to authenticate on subsequent visits. A WebAuthn assertion ceremony (challenge → biometric sign → verify) and an embedded login page are needed to issue session cookies to returning users. See `initial_specs/DESIGN-users.md` point 5.

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
