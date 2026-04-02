+++
id = "e35ef349"
title = "CLI: apm register, apm sessions, and apm revoke commands"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/e35ef349-cli-apm-register-apm-sessions-and-apm-re"
created_at = "2026-04-02T20:54:25.629052Z"
updated_at = "2026-04-02T23:22:59.884235Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["8a08637c"]
+++

## Spec

### Problem

There are no CLI commands for managing the WebAuthn auth lifecycle: generating OTPs to bootstrap device registration (`apm register`), inspecting active sessions (`apm sessions`), or revoking compromised sessions (`apm revoke`). See `initial_specs/DESIGN-users.md` points 5 and 7.

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
