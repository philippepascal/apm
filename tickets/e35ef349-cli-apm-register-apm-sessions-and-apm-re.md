+++
id = "e35ef349"
title = "CLI: apm register, apm sessions, and apm revoke commands"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "36684"
branch = "ticket/e35ef349-cli-apm-register-apm-sessions-and-apm-re"
created_at = "2026-04-02T20:54:25.629052Z"
updated_at = "2026-04-02T23:59:31.321375Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["8a08637c"]
+++

## Spec

### Problem

The APM server supports WebAuthn-based authentication for remote collaborators, but there is no CLI surface to operate the auth lifecycle. Specifically:

- **Device registration**: a new device must be bootstrapped with a one-time password generated server-side. Without `apm register`, a project admin has no way to produce that OTP from the command line; they would need to call the raw HTTP endpoint manually.
- **Session visibility**: there is no way to inspect which devices hold active sessions, making it impossible to audit access or detect stale/compromised tokens.
- **Session revocation**: there is no way to invalidate a session. If a device is lost or a token is leaked, the only recourse is to wait for the 7-day TTL to expire.

The desired behaviour is three new subcommands — `apm register`, `apm sessions`, and `apm revoke` — that let a localhost-connected admin drive the full auth lifecycle without leaving the terminal. The server-side list and revoke endpoints also do not yet exist and must be added as part of this ticket.

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
| 2026-04-02T23:59Z | groomed | in_design | philippepascal |