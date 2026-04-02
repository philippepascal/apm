+++
id = "70d58b2d"
title = "apm-server: WebAuthn login/assertion ceremony and embedded login page"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "21293"
branch = "ticket/70d58b2d-apm-server-webauthn-login-assertion-cere"
created_at = "2026-04-02T20:54:21.301151Z"
updated_at = "2026-04-02T23:56:27.321648Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["8a08637c"]
+++

## Spec

### Problem

Registered devices have no way to authenticate on subsequent visits to apm-server. Once a user has enrolled a passkey via the registration ceremony (ticket 8a08637c), there is no endpoint that issues a session cookie to a returning device. Without a WebAuthn assertion ceremony and a matching login page, every visit after the first requires re-registration, which defeats the purpose of passkeys.

The desired behaviour: a returning browser that holds a registered passkey visits apm-server, sees a login page, enters their username, completes a biometric prompt, and receives a session cookie granting access. The server verifies the signed challenge against the stored public key — no shared secret is transmitted.

External devices (phone, remote laptop) are the primary audience. Localhost requests bypass auth entirely (established in ticket e2e3d958) so the login flow is only reachable by non-loopback clients.

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
| 2026-04-02T23:56Z | groomed | in_design | philippepascal |