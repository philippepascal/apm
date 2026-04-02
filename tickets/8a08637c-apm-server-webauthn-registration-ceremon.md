+++
id = "8a08637c"
title = "apm-server: WebAuthn registration ceremony and embedded registration page"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/8a08637c-apm-server-webauthn-registration-ceremon"
created_at = "2026-04-02T20:54:17.589009Z"
updated_at = "2026-04-02T23:22:51.104439Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["e2e3d958"]
+++

## Spec

### Problem

There is no registration flow for external devices to enroll a passkey with apm-server. Without a WebAuthn registration ceremony and an embedded registration page, devices cannot authenticate. The OTP from `apm register` serves as the trust gate for this ceremony. See `initial_specs/DESIGN-users.md` point 5.

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
