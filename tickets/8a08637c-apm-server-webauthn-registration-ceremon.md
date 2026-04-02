+++
id = "8a08637c"
title = "apm-server: WebAuthn registration ceremony and embedded registration page"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "12166"
branch = "ticket/8a08637c-apm-server-webauthn-registration-ceremon"
created_at = "2026-04-02T20:54:17.589009Z"
updated_at = "2026-04-02T23:51:42.897497Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["e2e3d958"]
+++

## Spec

### Problem

There is no registration flow for external devices to enroll a passkey with apm-server. Ticket e2e3d958 (dependency) provides the OTP infrastructure, session store, and localhost bypass — the trust foundation. Without the WebAuthn registration ceremony and an embedded HTML registration page, a device that has received an OTP cannot complete enrollment and cannot authenticate at all.

The desired behaviour: a device visits apm-server in a browser, sees a registration page, enters username + OTP, and completes a biometric-secured WebAuthn credential enrollment. After registration, the device holds a session cookie and can access apm-server.

External devices (phone, remote laptop) are the affected audience. The OTP from `apm register` is the sole trust gate for the ceremony — it is never itself a persistent credential.

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
| 2026-04-02T23:51Z | groomed | in_design | philippepascal |